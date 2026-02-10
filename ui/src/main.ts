import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

const app = mount(App, {
  target: document.getElementById('app')!,
})

const SCROLL_SPEED_MULTIPLIER = 1.5;
const LERP_FACTOR = 0.14;
const STOP_THRESHOLD = 0.5;

function findScrollableAncestor(el: Element | null): Element | null {
  while (el) {
    if (el === document.documentElement || el === document.body) {
      if (el.scrollHeight > el.clientHeight) return el;
      return null;
    }
    const style = getComputedStyle(el);
    const overflowY = style.overflowY;
    if (
      (overflowY === 'auto' || overflowY === 'scroll') &&
      el.scrollHeight > el.clientHeight
    ) {
      return el;
    }
    el = el.parentElement;
  }
  return null;
}

let pendingDelta = 0;
let activeScroller: Element | null = null;
let animationId = 0;

function tick() {
  if (!activeScroller || Math.abs(pendingDelta) < STOP_THRESHOLD) {
    pendingDelta = 0;
    activeScroller = null;
    animationId = 0;
    return;
  }

  const step = pendingDelta * LERP_FACTOR;
  pendingDelta -= step;
  activeScroller.scrollBy({ top: step });

  animationId = requestAnimationFrame(tick);
}

document.addEventListener(
  'wheel',
  (e: WheelEvent) => {
    if (e.deltaY === 0 || e.ctrlKey) return;

    const scroller = findScrollableAncestor(e.target as Element);
    if (!scroller) return;

    e.preventDefault();

    if (activeScroller !== scroller) {
      pendingDelta = 0;
      activeScroller = scroller;
    }

    pendingDelta += e.deltaY * SCROLL_SPEED_MULTIPLIER;

    if (!animationId) {
      animationId = requestAnimationFrame(tick);
    }
  },
  { passive: false },
);

export default app
