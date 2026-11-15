import { chooseOs, detectOs, type Detected, type Os } from '../lib/platform';

const OS_KEY = 'framely.os';
const LANG_KEY = 'framely.lang';
const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');

function store(key: string, value?: string): string | null {
  try {
    if (value !== undefined) localStorage.setItem(key, value);
    return localStorage.getItem(key);
  } catch {
    return null; // private mode / blocked storage: the page still works, without memory
  }
}

// ── Header: background once scrolled, mobile menu, language menu ─────────────
const header = document.querySelector<HTMLElement>('[data-header]');
const onScroll = () => header?.classList.toggle('scrolled', window.scrollY > 8);
onScroll();
window.addEventListener('scroll', onScroll, { passive: true });

const menuBtn = document.querySelector<HTMLButtonElement>('[data-menu]');
const menuLabel = document.querySelector<HTMLElement>('[data-menu-label]');
function setMenu(open: boolean) {
  if (!header || !menuBtn) return;
  header.classList.toggle('open', open);
  menuBtn.setAttribute('aria-expanded', String(open));
  if (menuLabel) menuLabel.textContent = (open ? menuBtn.dataset.close : menuBtn.dataset.open) ?? '';
  document.documentElement.style.overflow = open ? 'hidden' : '';
}
menuBtn?.addEventListener('click', () => setMenu(!header?.classList.contains('open')));
document.querySelectorAll('[data-nav] a').forEach((a) => a.addEventListener('click', () => setMenu(false)));
window.matchMedia('(min-width: 921px)').addEventListener('change', (e) => e.matches && setMenu(false));

const lang = document.querySelector<HTMLDetailsElement>('[data-lang]');
document.addEventListener('click', (e) => {
  if (lang?.open && !lang.contains(e.target as Node)) lang.open = false;
});
document.addEventListener('keydown', (e) => {
  if (e.key !== 'Escape') return;
  if (lang?.open) {
    lang.open = false;
    lang.querySelector('summary')?.focus();
  }
  if (header?.classList.contains('open')) {
    setMenu(false);
    menuBtn?.focus();
  }
});
document.querySelectorAll<HTMLAnchorElement>('[data-set-lang]').forEach((a) =>
  a.addEventListener('click', () => store(LANG_KEY, a.dataset.setLang)),
);

// ── Reveal on scroll ────────────────────────────────────────────────────────
const revealables = document.querySelectorAll<HTMLElement>('[data-reveal]');
if ('IntersectionObserver' in window && !reduced.matches) {
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (e.isIntersecting) {
          e.target.classList.add('in');
          io.unobserve(e.target);
        }
      }
    },
    { rootMargin: '0px 0px -8% 0px', threshold: 0.08 },
  );
  revealables.forEach((el) => io.observe(el));
} else {
  revealables.forEach((el) => el.classList.add('in'));
}

// ── Hero screenshot: flattens as it scrolls into view ───────────────────────
const tilt = document.querySelector<HTMLElement>('[data-tilt]');
if (tilt && !reduced.matches) {
  let ticking = false;
  const update = () => {
    ticking = false;
    const r = tilt.getBoundingClientRect();
    const p = Math.min(1, Math.max(0, 1 - (r.top - window.innerHeight * 0.18) / (window.innerHeight * 0.62)));
    tilt.style.setProperty('--tilt', `${(14 * (1 - p)).toFixed(2)}deg`);
    tilt.style.setProperty('--scale', (0.94 + 0.06 * p).toFixed(4));
  };
  update();
  window.addEventListener(
    'scroll',
    () => {
      if (!ticking) {
        ticking = true;
        requestAnimationFrame(update);
      }
    },
    { passive: true },
  );
}

// ── Operating system: detection, manual choice (remembered), download UI ────
const detected: Detected = detectOs(navigator as Navigator & { userAgentData?: { platform?: string; mobile?: boolean } }, navigator.maxTouchPoints);

function applyOs(choice: Detected) {
  const os: Os | null = choice === 'other' ? null : choice;
  // Hero button: direct link to the real installer when there is one, otherwise the download section.
  const hero = document.querySelector<HTMLAnchorElement>('[data-hero-download]');
  if (hero && os) {
    const target = hero.dataset[`href${os === 'macos' ? 'Macos' : 'Windows'}`] ?? '#download';
    hero.href = target;
    if (target.startsWith('http')) hero.setAttribute('download', '');
    else hero.removeAttribute('download');
    const label = hero.querySelector('[data-label]');
    if (label) label.textContent = hero.dataset[`label${os === 'macos' ? 'Macos' : 'Windows'}`] ?? label.textContent;
  }
  const det = document.querySelector<HTMLElement>('[data-detected]');
  if (det) det.textContent = det.dataset[`text${detected === 'macos' ? 'Macos' : detected === 'windows' ? 'Windows' : 'Other'}`] ?? '';
  const ver = document.querySelector<HTMLElement>('[data-hero-version]');
  if (ver && os) {
    const v = ver.dataset[`version${os === 'macos' ? 'Macos' : 'Windows'}`];
    if (v) ver.textContent = (ver.dataset.template ?? '{version}').replace('{version}', v);
  }
  // Download section.
  const box = document.querySelector<HTMLElement>('[data-download]');
  if (!box) return;
  box.classList.toggle('chosen', !!os);
  box.querySelectorAll<HTMLElement>('[data-os-panel]').forEach((p) => p.classList.toggle('active', p.dataset.osPanel === os));
  box.querySelectorAll<HTMLButtonElement>('[data-os-choice]').forEach((b) => {
    const on = b.dataset.osChoice === os;
    b.setAttribute('aria-checked', String(on));
    b.tabIndex = on || (!os && b.dataset.osChoice === 'macos') ? 0 : -1;
  });
  const hint = box.querySelector<HTMLElement>('[data-os-hint]');
  if (hint) hint.textContent = hint.dataset[`text${detected === 'macos' ? 'Macos' : detected === 'windows' ? 'Windows' : 'Other'}`] ?? '';
}

applyOs(chooseOs(store(OS_KEY), detected));

const choices = Array.from(document.querySelectorAll<HTMLButtonElement>('[data-os-choice]'));
choices.forEach((b, i) => {
  b.addEventListener('click', () => {
    const os = b.dataset.osChoice as Os;
    store(OS_KEY, os);
    applyOs(os);
  });
  // Radio group keyboard behaviour: arrows move and select.
  b.addEventListener('keydown', (e) => {
    if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(e.key)) return;
    e.preventDefault();
    const next = choices[(i + (e.key === 'ArrowLeft' || e.key === 'ArrowUp' ? -1 : 1) + choices.length) % choices.length];
    next?.click();
    next?.focus();
  });
});

// ── Demo video: plays only while visible, never with reduced motion ─────────
const player = document.querySelector<HTMLElement>('[data-player]');
const video = player?.querySelector<HTMLVideoElement>('[data-video]');
const toggle = player?.querySelector<HTMLButtonElement>('[data-toggle]');
if (video && toggle) {
  toggle.hidden = false;
  let userPaused = reduced.matches;
  const sync = () => toggle.setAttribute('aria-pressed', String(!video.paused));
  video.addEventListener('play', sync);
  video.addEventListener('pause', sync);
  toggle.addEventListener('click', () => {
    if (video.paused) {
      userPaused = false;
      void video.play().catch(() => {});
    } else {
      userPaused = true;
      video.pause();
    }
  });
  if ('IntersectionObserver' in window) {
    new IntersectionObserver(
      ([e]) => {
        if (e?.isIntersecting && !userPaused) void video.play().catch(() => {});
        else if (!e?.isIntersecting && !video.paused) video.pause();
      },
      { threshold: 0.5 },
    ).observe(video);
  }
}

// ── Current section in the header ───────────────────────────────────────────
const navLinks = Array.from(document.querySelectorAll<HTMLAnchorElement>('[data-navlink]')).filter((a) => a.getAttribute('href')?.startsWith('#'));
if (navLinks.length && 'IntersectionObserver' in window) {
  const map = new Map(navLinks.map((a) => [a.getAttribute('href')!.slice(1), a]));
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        navLinks.forEach((a) => a.removeAttribute('aria-current'));
        map.get(e.target.id)?.setAttribute('aria-current', 'true');
      }
    },
    { rootMargin: '-45% 0px -50% 0px' },
  );
  map.forEach((_, id) => {
    const el = document.getElementById(id);
    if (el) io.observe(el);
  });
}
