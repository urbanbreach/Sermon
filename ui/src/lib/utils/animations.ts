/**
 * Animation Utilities - Spring Physics for Cider-style UI
 * 
 * Provides reusable spring configurations and CSS custom properties
 * for consistent, fluid animations throughout the app.
 */

// Spring presets matching Cider's motion design
export const springPresets = {
  // Snappy interactions (buttons, toggles)
  snappy: { stiffness: 400, damping: 30, mass: 1 },
  
  // Standard UI transitions (panels, modals)
  standard: { stiffness: 300, damping: 25, mass: 1 },
  
  // Gentle, smooth transitions (backgrounds, large elements)
  gentle: { stiffness: 200, damping: 20, mass: 1 },
  
  // Bouncy emphasis (notifications, success states)
  bouncy: { stiffness: 500, damping: 15, mass: 1 },
  
  // Slow, elegant (page transitions, artwork reveals)
  elegant: { stiffness: 150, damping: 25, mass: 1.2 },
} as const;

export type SpringPreset = keyof typeof springPresets;

/**
 * CSS timing functions that approximate spring physics
 * Use these in CSS transitions when JS springs aren't needed
 */
export const easings = {
  // Standard ease - smooth start and end
  standard: 'cubic-bezier(0.4, 0, 0.2, 1)',
  
  // Emphasis - slight overshoot for playful feel
  emphasis: 'cubic-bezier(0.175, 0.885, 0.32, 1.275)',
  
  // Enter - elements appearing (fade in, scale up)
  enter: 'cubic-bezier(0, 0, 0.2, 1)',
  
  // Exit - elements disappearing (fade out, scale down)
  exit: 'cubic-bezier(0.4, 0, 1, 1)',
  
  // Spring approximation - bouncy feel
  spring: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
  
  // Smooth - very gentle, no acceleration feel
  smooth: 'cubic-bezier(0.25, 0.1, 0.25, 1)',
} as const;

export type Easing = keyof typeof easings;

/**
 * Duration presets in milliseconds
 */
export const durations = {
  instant: 100,
  fast: 150,
  medium: 200,
  slow: 300,
  slower: 400,
  slowest: 500,
} as const;

export type Duration = keyof typeof durations;

/**
 * Generate CSS transition string
 */
export function createTransition(
  properties: string | string[],
  duration: Duration | number = 'medium',
  easing: Easing | string = 'standard'
): string {
  const props = Array.isArray(properties) ? properties : [properties];
  const dur = typeof duration === 'number' ? duration : durations[duration];
  const ease = easings[easing as Easing] || easing;
  
  return props.map(prop => `${prop} ${dur}ms ${ease}`).join(', ');
}

/**
 * Stagger delay calculator for list animations
 */
export function staggerDelay(index: number, baseDelay: number = 50, maxDelay: number = 300): number {
  return Math.min(index * baseDelay, maxDelay);
}

/**
 * Svelte action for hover scale effect
 * Usage: <div use:hoverScale={{ scale: 1.05, duration: 200 }}>
 */
export function hoverScale(node: HTMLElement, params: { scale?: number; duration?: number } = {}) {
  const { scale = 1.03, duration = 200 } = params;
  
  function isReduced() {
    return typeof document !== 'undefined' && document.documentElement.classList.contains('reduce-effects');
  }
  
  const originalTransform = node.style.transform || '';
  const originalTransition = node.style.transition || '';
  
  function handleMouseEnter() {
    if (isReduced()) return;
    node.style.transition = `transform ${duration}ms ${easings.emphasis}`;
    node.style.transform = `${originalTransform} scale(${scale})`.trim();
  }
  
  function handleMouseLeave() {
    node.style.transition = `transform ${duration}ms ${easings.standard}`;
    node.style.transform = originalTransform || 'scale(1)';
  }
  
  node.addEventListener('mouseenter', handleMouseEnter);
  node.addEventListener('mouseleave', handleMouseLeave);
  
  return {
    destroy() {
      node.removeEventListener('mouseenter', handleMouseEnter);
      node.removeEventListener('mouseleave', handleMouseLeave);
      node.style.transform = originalTransform;
      node.style.transition = originalTransition;
    },
    update(newParams: { scale?: number; duration?: number }) {
      Object.assign(params, newParams);
    }
  };
}

/**
 * Svelte action for press/active scale effect
 * Usage: <button use:pressScale={{ scale: 0.95 }}>
 */
export function pressScale(node: HTMLElement, params: { scale?: number; duration?: number } = {}) {
  const { scale = 0.95, duration = 150 } = params;

  function isReduced() {
    return typeof document !== 'undefined' && document.documentElement.classList.contains('reduce-effects');
  }
  
  function handleMouseDown() {
    if (isReduced()) return;
    node.style.transition = `transform ${duration}ms ${easings.standard}`;
    node.style.transform = `scale(${scale})`;
  }
  
  function handleMouseUp() {
    node.style.transition = `transform ${duration}ms ${easings.emphasis}`;
    node.style.transform = 'scale(1)';
  }
  
  node.addEventListener('mousedown', handleMouseDown);
  node.addEventListener('mouseup', handleMouseUp);
  node.addEventListener('mouseleave', handleMouseUp);
  
  // Reduced motion handling
  if (typeof window !== 'undefined') {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    
    function updateReducedMotion() {
      if (mediaQuery.matches) {
        document.documentElement.classList.add('reduce-effects');
      } else {
        // Only remove if it was added by media query, but for simplicity:
        // document.documentElement.classList.remove('reduce-effects');
      }
    }
    
    mediaQuery.addEventListener('change', updateReducedMotion);
    updateReducedMotion();
  }
  
  return {
    destroy() {
      node.removeEventListener('mousedown', handleMouseDown);
      node.removeEventListener('mouseup', handleMouseUp);
      node.removeEventListener('mouseleave', handleMouseUp);
    }
  };
}

/**
 * Svelte action for fade-in on mount
 * Usage: <div use:fadeIn={{ duration: 300, delay: 100 }}>
 */
export function fadeIn(node: HTMLElement, params: { duration?: number; delay?: number; y?: number } = {}) {
  const { duration = 300, delay = 0, y = 10 } = params;
  
  node.style.opacity = '0';
  node.style.transform = `translateY(${y}px)`;
  
  requestAnimationFrame(() => {
    setTimeout(() => {
      node.style.transition = `opacity ${duration}ms ${easings.enter}, transform ${duration}ms ${easings.enter}`;
      node.style.opacity = '1';
      node.style.transform = 'translateY(0)';
    }, delay);
  });
  
  return {
    destroy() {
      // Cleanup if needed
    }
  };
}

/**
 * Apply staggered fade-in to list items
 * Usage: <li use:staggeredFadeIn={{ index: i, baseDelay: 50 }}>
 */
export function staggeredFadeIn(node: HTMLElement, params: { index: number; baseDelay?: number; duration?: number }) {
  const { index, baseDelay = 50, duration = 300 } = params;
  const delay = staggerDelay(index, baseDelay);
  
  node.style.opacity = '0';
  node.style.transform = 'translateY(8px)';
  
  requestAnimationFrame(() => {
    setTimeout(() => {
      node.style.transition = `opacity ${duration}ms ${easings.enter}, transform ${duration}ms ${easings.enter}`;
      node.style.opacity = '1';
      node.style.transform = 'translateY(0)';
    }, delay);
  });
  
  return {
    destroy() {}
  };
}
