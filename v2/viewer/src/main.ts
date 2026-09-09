import './style.css';
import { type Encounter, type Fight, type Run, demoFight, demoRun, parseFight, parseRun } from './data';
import { RECORD_DEFAULT, applyRecordMode, isRecordMode } from './record';
import { Scene } from './scene';
import { DEFAULT_ACCENT, type Theme, applyTheme, nextTheme, setAccent } from './theme';
