//! Imitation: distil a scripted style teacher into a net (supervised, from the net's own sensed inputs).
//! The loss is a softmax cross-entropy over the 14 move outputs plus squared error on the four move params (only when
//! a move is asked for) and on the footwork drive. The recurrent context is carried through each fight in order and
//! treated as a constant input (truncated backprop). Later rounds (DAgger) let the current student fight while the
//! teacher watches and labels the states the student actually reaches.
use crate::arena::Arena;
use crate::duel::{Brain, Match, Sample};
use crate::genome::Genome;
use crate::net::{fast_tanh, N_CTX, N_HID, N_OUT, N_PARAMS};
use crate::moveset::N_MOVES;
use crate::rng::Rng;
use crate::robot::Role;
use crate::sense::N_IN;
use crate::stylebot::StyleId;

/// Softmax temperature on the tanh move outputs.
const K_LOGIT: f32 = 4.0;
/// Extra weight on samples that ask for a move (Idle dominates the data).
const W_MOVE: f32 = 2.5;
const W_PARAM: f32 = 1.0;
const W_DRIVE: f32 = 1.0;
/// Longest fight sequence kept (decision steps).
const MAX_SEQ: usize = 1200;

/// Per-input standardisation used while fitting; folded into the first layer afterwards so the net takes raw inputs.
#[derive(Clone)]
pub struct Norm {
    mu: Vec<f32>,
    sd: Vec<f32>,
}

impl Norm {
    pub fn from_data(d: &Dataset) -> Norm {
        let (mut sum, mut sq, mut n) = (vec![0.0f64; N_IN], vec![0.0f64; N_IN], 0.0f64);
        for seq in &d.seqs {
            for s in seq {
                for i in 0..N_IN {
                    let v = s.x[i] as f64;
                    sum[i] += v;
                    sq[i] += v * v;
                }
                n += 1.0;
            }
        }
        let n = n.max(1.0);
        let mu: Vec<f32> = sum.iter().map(|v| (v / n) as f32).collect();
        let sd: Vec<f32> = (0..N_IN).map(|i| (((sq[i] / n) - (sum[i] / n).powi(2)).max(0.0).sqrt() as f32).max(SD_FLOOR)).collect();
        Norm { mu, sd }
    }
    fn apply(&self, x: &[f32]) -> [f32; N_IN] {
        let mut o = [0.0f32; N_IN];
        for i in 0..N_IN {
            o[i] = (x[i] - self.mu[i]) / self.sd[i];
        }
        o
    }
    /// Weights acting on raw inputs that reproduce `w` acting on standardised inputs.
    pub fn fold(&self, w: &[f32]) -> Vec<f32> {
        let mut o = w.to_vec();
        let n1 = w1_len();
        for j in 0..N_HID {
            let base = j * (N_IN + N_CTX);
            let mut shift = 0.0;
            for i in 0..N_IN {
                let wi = w[base + i] / self.sd[i];
                o[base + i] = wi;
                shift += wi * self.mu[i];
            }
            o[n1 + j] -= shift;
        }
        o
    }
}

/// Smallest input spread used when standardising (keeps folded weights bounded).
const SD_FLOOR: f32 = 0.03;

pub struct Dataset {
    pub seqs: Vec<Vec<Sample>>,
}

impl Dataset {
    pub fn len(&self) -> usize {
        self.seqs.iter().map(|s| s.len()).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.seqs.is_empty()
    }
    pub fn extend(&mut self, o: Dataset) {
        self.seqs.extend(o.seqs);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FitStats {
    pub loss: f32,
    /// Share of samples whose argmax move (with the net's Idle margin) equals the teacher's.
    pub move_acc: f32,
    /// The same, over the samples where the teacher asks for a move.
    pub active_acc: f32,
    pub samples: usize,
}

fn opponents() -> Vec<(Brain, bool)> {
    // (brain, flat arena): a standing dummy cannot be reached across obstacles, so it meets the teacher on flat ground.
    let mut v = vec![(Brain::Dummy, true), (Brain::Skill(1), false), (Brain::Skill(2), false), (Brain::Skill(3), false)];
    for s in StyleId::ALL {
        v.push((Brain::Style(s), false));
    }
    v
}

struct Job {
    me: [Brain; 2],
    opp: Brain,
    flat: bool,
    seed: u64,
    side: usize,
}

fn run_jobs(jobs: &[Job]) -> Vec<Vec<Sample>> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    let n = jobs.len();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).min(n.max(1));
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<Vec<Sample>>>> = Mutex::new((0..n).map(|_| None).collect());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= n {
                    break;
                }
                let j = &jobs[i];
                let (a, b) = if j.side == 0 { (j.me[0].clone(), j.opp.clone()) } else { (j.opp.clone(), j.me[1].clone()) };
                let mut m = Match::new(j.seed, a, b);
                if j.flat {
                    m.arena = Arena::flat(m.arena.width);
                }
                m.disable_grace_trace();
                m.recorder = Some(crate::duel::Recorder { side: j.side, samples: Vec::new() });
                m.run_to_end();
                let mut samples = m.recorder.take().unwrap().samples;
                samples.truncate(MAX_SEQ);
                for s in samples.iter_mut() {
                    if s.n > 0 {
                        s.drive /= s.n as f32;
                    }
                }
                out.lock().unwrap()[i] = Some(samples);
            });
        }
    });
    out.into_inner().unwrap().into_iter().map(|o| o.expect("job")).collect()
}

fn collect(me: [Brain; 2], per_opp: usize, seed: u64) -> Dataset {
    let mut jobs = Vec::new();
    for (k, (opp, flat)) in opponents().into_iter().enumerate() {
        for i in 0..per_opp {
            jobs.push(Job { me: me.clone(), opp: opp.clone(), flat, seed: seed.wrapping_add((k * 1000 + i) as u64 * 7919), side: i % 2 });
        }
    }
    Dataset { seqs: run_jobs(&jobs).into_iter().filter(|s| !s.is_empty()).collect() }
}

/// Fights of the teacher itself against the opponent pool (`per_opp` fights per opponent, alternating slots).
pub fn collect_teacher(style: StyleId, per_opp: usize, seed: u64) -> Dataset {
    collect([Brain::Style(style), Brain::Style(style)], per_opp, seed)
}

/// Fights of a student net with the teacher shadowing it (DAgger round).
pub fn collect_student(style: StyleId, student: &Genome, per_opp: usize, seed: u64) -> Dataset {
    collect([Brain::Student(student.mirrored_drive(Role::Hero), style), Brain::Student(student.mirrored_drive(Role::Monster), style)], per_opp, seed)
}

fn w1_len() -> usize {
    N_HID * (N_IN + N_CTX)
}

pub fn init_weights(rng: &mut Rng) -> Vec<f32> {
    let mut w = vec![0.0f32; N_PARAMS];
    let s1 = (1.0 / (N_IN + N_CTX) as f32).sqrt() * 1.7;
    let s2 = (1.0 / N_HID as f32).sqrt() * 1.7;
    let n1 = w1_len();
    for v in w[..n1].iter_mut() {
        *v = rng.range(-s1, s1);
    }
    for v in w[n1 + N_HID..n1 + N_HID + N_OUT * N_HID].iter_mut() {
        *v = rng.range(-s2, s2);
    }
    w
}

struct Fwd {
    h: [f32; N_HID],
    out: [f32; N_OUT],
}

fn forward(w: &[f32], x: &[f32; N_IN], ctx: &[f32; N_CTX]) -> Fwd {
    let n1 = w1_len();
    let (w1, rest) = w.split_at(n1);
    let (b1, rest) = rest.split_at(N_HID);
    let (w2, b2) = rest.split_at(N_OUT * N_HID);
    let mut h = [0.0f32; N_HID];
    for j in 0..N_HID {
        let row = &w1[j * (N_IN + N_CTX)..(j + 1) * (N_IN + N_CTX)];
        let mut s = b1[j];
        for i in 0..N_IN {
            s += row[i] * x[i];
        }
        for k in 0..N_CTX {
            s += row[N_IN + k] * ctx[k];
        }
        h[j] = fast_tanh(s);
    }
    let mut out = [0.0f32; N_OUT];
    for o in 0..N_OUT {
        let row = &w2[o * N_HID..(o + 1) * N_HID];
        let mut s = b2[o];
        for j in 0..N_HID {
            s += row[j] * h[j];
        }
        out[o] = fast_tanh(s);
    }
    Fwd { h, out }
}

fn decode_mv(out: &[f32; N_OUT]) -> usize {
    let mut best = 0;
    for i in 1..N_MOVES {
        if out[i] > out[best] {
            best = i;
        }
    }
    if best != 0 && out[best] > out[0] + crate::net::MOVE_MARGIN { best } else { 0 }
}

/// Loss of one sample and its gradient w.r.t. the 19 outputs.
fn loss_and_dout(out: &[f32; N_OUT], s: &Sample) -> (f32, [f32; N_OUT]) {
    let mut g = [0.0f32; N_OUT];
    let mv = s.mv as usize;
    let wcls = if mv != 0 { W_MOVE } else { 1.0 };
    let mut mx = f32::MIN;
    for i in 0..N_MOVES {
        mx = mx.max(K_LOGIT * out[i]);
    }
    let mut z = 0.0;
    let mut e = [0.0f32; N_MOVES];
    for i in 0..N_MOVES {
        e[i] = (K_LOGIT * out[i] - mx).exp();
        z += e[i];
    }
    let mut loss = wcls * (z.ln() + mx - K_LOGIT * out[mv]);
    for i in 0..N_MOVES {
        let p = e[i] / z;
        g[i] = wcls * K_LOGIT * (p - if i == mv { 1.0 } else { 0.0 });
    }
    if mv != 0 {
        for k in 0..4 {
            let t = s.params[k] * 2.0 - 1.0;
            let d = out[14 + k] - t;
            loss += W_PARAM * d * d;
            g[14 + k] = W_PARAM * 2.0 * d;
        }
    }
    let t = s.drive.clamp(-1.0, 1.0);
    let d = out[18] - t;
    loss += W_DRIVE * d * d;
    g[18] = W_DRIVE * 2.0 * d;
    (loss, g)
}

pub struct FitOpts {
    pub epochs: usize,
    pub lr: f32,
    pub seed: u64,
}

/// Adam over fight sequences (one update per sequence, sequences shuffled each epoch).
pub fn fit(w: &mut Vec<f32>, data: &Dataset, norm: &Norm, opts: &FitOpts) -> FitStats {
    let n = w.len();
    let (mut m1, mut m2) = (vec![0.0f32; n], vec![0.0f32; n]);
    let mut grad = vec![0.0f32; n];
    let mut rng = Rng::new(opts.seed);
    let mut order: Vec<usize> = (0..data.seqs.len()).collect();
    let (b1c, b2c, eps) = (0.9f32, 0.999f32, 1e-8f32);
    let mut t = 0i32;
    let n1 = w1_len();
    let total = (opts.epochs * data.seqs.len()).max(1);
    let mut done = 0usize;
    for _ in 0..opts.epochs {
        for i in (1..order.len()).rev() {
            let j = (rng.next_u64() % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        for &si in &order {
            let seq = &data.seqs[si];
            grad.iter_mut().for_each(|g| *g = 0.0);
            let mut ctx = [0.0f32; N_CTX];
            for s in seq {
                let xn = norm.apply(&s.x);
                let f = forward(w, &xn, &ctx);
                let (_, dout) = loss_and_dout(&f.out, s);
                // Backward: output layer.
                let mut dh = [0.0f32; N_HID];
                let w2_off = n1 + N_HID;
                let b2_off = w2_off + N_OUT * N_HID;
                for o in 0..N_OUT {
                    let dz = dout[o] * (1.0 - f.out[o] * f.out[o]);
                    if dz == 0.0 {
                        continue;
                    }
                    grad[b2_off + o] += dz;
                    for j in 0..N_HID {
                        grad[w2_off + o * N_HID + j] += dz * f.h[j];
                        dh[j] += dz * w[w2_off + o * N_HID + j];
                    }
                }
                for j in 0..N_HID {
                    let dz = dh[j] * (1.0 - f.h[j] * f.h[j]);
                    if dz == 0.0 {
                        continue;
                    }
                    grad[n1 + j] += dz;
                    let base = j * (N_IN + N_CTX);
                    for i in 0..N_IN {
                        grad[base + i] += dz * xn[i];
                    }
                    for k in 0..N_CTX {
                        grad[base + N_IN + k] += dz * ctx[k];
                    }
                }
                ctx.copy_from_slice(&f.h[..N_CTX]);
            }
            let inv = 1.0 / seq.len().max(1) as f32;
            t += 1;
            // Cosine-ish decay of the learning rate over the whole fit.
            let frac = done as f32 / total as f32;
            let lr = opts.lr * (0.1 + 0.9 * 0.5 * (1.0 + (std::f32::consts::PI * frac).cos()));
            done += 1;
            let (c1, c2) = (1.0 - b1c.powi(t), 1.0 - b2c.powi(t));
            for k in 0..n {
                let g = grad[k] * inv;
                m1[k] = b1c * m1[k] + (1.0 - b1c) * g;
                m2[k] = b2c * m2[k] + (1.0 - b2c) * g * g;
                w[k] = (w[k] - lr * (m1[k] / c1) / ((m2[k] / c2).sqrt() + eps)).clamp(-crate::genome::W_CLAMP, crate::genome::W_CLAMP);
            }
        }
    }
    evaluate(w, data, norm)
}

/// Loss and accuracy of the weights on a dataset.
pub fn evaluate(w: &[f32], data: &Dataset, norm: &Norm) -> FitStats {
    let (mut loss, mut ok, mut act, mut act_ok, mut cnt) = (0.0f64, 0usize, 0usize, 0usize, 0usize);
    for seq in &data.seqs {
        let mut ctx = [0.0f32; N_CTX];
        for s in seq {
            let xn = norm.apply(&s.x);
            let f = forward(w, &xn, &ctx);
            loss += loss_and_dout(&f.out, s).0 as f64;
            let hit = decode_mv(&f.out) == s.mv as usize;
            ok += hit as usize;
            if s.mv != 0 {
                act += 1;
                act_ok += hit as usize;
            }
            cnt += 1;
            ctx.copy_from_slice(&f.h[..N_CTX]);
        }
    }
    FitStats { loss: (loss / cnt.max(1) as f64) as f32, move_acc: ok as f32 / cnt.max(1) as f32, active_acc: act_ok as f32 / act.max(1) as f32, samples: cnt }
}
