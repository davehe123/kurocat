import './controlPanel.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { KuroCatEngine } from './catEngine';
import type { CatSkinConfig } from './catSkins';
import { saveDesktopSettings, type AppSettings, type SettingsPatch } from './desktopSettings';

function element<T extends HTMLElement>(id: string): T {
  const node = document.getElementById(id);
  if (!node) throw new Error('Missing control: ' + id);
  return node as T;
}

const canvas = element<HTMLCanvasElement>('preview');
const preview = new KuroCatEngine(canvas, { embedded: true });
const role = element<HTMLSelectElement>('role');
const scale = element<HTMLInputElement>('scale');
const scaleValue = element<HTMLOutputElement>('scale-value');
const layout = element<HTMLSelectElement>('layout');
const orientation = element<HTMLSelectElement>('orientation');
const top = element<HTMLInputElement>('top');
const sound = element<HTMLInputElement>('sound');
const bubbles = element<HTMLInputElement>('bubbles');
const privacy = element<HTMLInputElement>('privacy');
const status = element<HTMLParagraphElement>('status');
const name = element<HTMLInputElement>('skin-name');
const fur = element<HTMLInputElement>('fur');
const eyes = element<HTMLInputElement>('eyes');
const accent = element<HTMLInputElement>('accent');
const save = element<HTMLButtonElement>('save-skin');
const show = element<HTMLButtonElement>('show-pet');
const typing = element<HTMLButtonElement>('try-typing');
const clicking = element<HTMLButtonElement>('try-clicking');

let catalog: CatSkinConfig[] = [];
let settings: AppSettings;
let settingsEventArrived = false;
let catsEventArrived = false;
let scaleDragging = false;
let typingTimer: number | undefined;
let clickingTimer: number | undefined;

function message(text: string, error = false): void {
  status.textContent = text;
  status.toggleAttribute('data-error', error);
}

function displayError(error: unknown): void {
  message(error instanceof Error ? error.message : String(error), true);
}

function setCatalog(cats: CatSkinConfig[]): void {
  catalog = cats;
  role.replaceChildren(...cats.map((cat) => new Option(cat.name, cat.id)));
}

function sync(): void {
  const selected = catalog.find((cat) => cat.id === settings.catBreed);
  role.value = selected?.id ?? '';
  element('active-role').textContent = selected?.name ?? '角色预览';
  if (!scaleDragging) {
    scale.value = String(Math.round(settings.scale * 100));
    scaleValue.value = scale.value + '%';
  }
  layout.value = settings.keyboardLayout;
  orientation.value = settings.keyboardOrientation;
  top.checked = settings.alwaysOnTop;
  sound.checked = settings.soundEnabled;
  bubbles.checked = settings.showKeyBubbles;
  privacy.checked = settings.privacyPasswordGuard;
  preview.setCatAppearance(settings.catBreed, selected);
  preview.setKeyboardConfig(settings.keyboardLayout, settings.keyboardOrientation);
  preview.setRenderScale(1);
  preview.showKeyBubbles = settings.showKeyBubbles;
}

async function change(patch: SettingsPatch): Promise<void> {
  try {
    settings = await saveDesktopSettings(patch);
    sync();
    message('已保存');
  } catch (error) {
    sync();
    displayError(error);
  }
}

function command(name: string, success: string): void {
  void invoke(name).then(() => message(success), displayError);
}

function connectControls(): void {
  role.addEventListener('change', () => void change({ catBreed: role.value }));
  scale.addEventListener('pointerdown', () => { scaleDragging = true; });
  window.addEventListener('pointerup', () => { scaleDragging = false; });
  window.addEventListener('pointercancel', () => { scaleDragging = false; sync(); });
  scale.addEventListener('input', () => { scaleValue.value = scale.value + '%'; });
  scale.addEventListener('change', () => {
    scaleDragging = false;
    void change({ scale: Number(scale.value) / 100 });
  });
  layout.addEventListener('change', () => void change({ keyboardLayout: layout.value === '108' ? '108' : '65' }));
  orientation.addEventListener('change', () => void change({ keyboardOrientation: orientation.value === 'viewer' ? 'viewer' : 'cat' }));
  top.addEventListener('change', () => void change({ alwaysOnTop: top.checked }));
  sound.addEventListener('change', () => void change({ soundEnabled: sound.checked }));
  bubbles.addEventListener('change', () => void change({ showKeyBubbles: bubbles.checked }));
  privacy.addEventListener('change', () => void change({ privacyPasswordGuard: privacy.checked }));
  show.addEventListener('click', () => command('show_desktop_pet', '桌宠已显示'));
  element('reset').addEventListener('click', () => command('reset_position', '桌宠已移回桌面右下角'));

  typing.addEventListener('click', () => {
    if (typingTimer !== undefined) return;
    preview.onKeyDown(0x46, false);
    typingTimer = window.setTimeout(() => {
      preview.onKeyUp(0x46);
      typingTimer = undefined;
    }, 130);
  });

  clicking.addEventListener('click', () => {
    if (clickingTimer !== undefined) return;
    preview.onMouseDown('left');
    clickingTimer = window.setTimeout(() => {
      preview.onMouseUp('left');
      clickingTimer = undefined;
    }, 130);
  });

  element<HTMLFormElement>('skin-form').addEventListener('submit', async (event) => {
    event.preventDefault();
    if (save.disabled) return;
    save.disabled = true;
    try {
      const cat = await invoke<CatSkinConfig>('save_custom_skin', {
        draft: { name: name.value, fur: fur.value, eyes: eyes.value, accent: accent.value },
      });
      const [cats, current] = await Promise.all([
        invoke<CatSkinConfig[]>('get_cats'),
        invoke<AppSettings>('get_settings'),
      ]);
      setCatalog(cats);
      settings = current;
      sync();
      name.value = '';
      message(cat.name + '已保存');
    } catch (error) {
      displayError(error);
    } finally {
      save.disabled = false;
    }
  });
}

async function start(): Promise<void> {
  await Promise.all([
    listen<AppSettings>('kuro-settings', ({ payload }) => {
      settingsEventArrived = true;
      settings = payload;
      sync();
    }),
    listen<CatSkinConfig[]>('kuro-cats', ({ payload }) => {
      catsEventArrived = true;
      setCatalog(payload);
      if (settings) sync();
    }),
  ]);

  const [loaded, cats] = await Promise.all([
    invoke<AppSettings>('get_settings'),
    invoke<CatSkinConfig[]>('get_cats'),
  ]);
  if (!settingsEventArrived) settings = loaded;
  if (!catsEventArrived) setCatalog(cats);
  sync();
  connectControls();
  for (const id of ['pet-settings', 'interaction-settings', 'skin-fields']) {
    element<HTMLFieldSetElement>(id).disabled = false;
  }
  show.disabled = typing.disabled = clicking.disabled = false;

  const updatePreview = () => {
    if (document.hidden) preview.stop();
    else { preview.resize(); preview.start(); }
  };
  document.addEventListener('visibilitychange', updatePreview);
  updatePreview();
}

void start().catch(displayError);
