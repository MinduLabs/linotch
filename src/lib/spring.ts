/** Damped spring with SwiftUI semantics: `response` is the period in seconds, `damping` the damping ratio. */
export type SpringCfg = { response: number; damping: number };

// boring.notch uses 0.42/0.8 (visible bounce). The tail of that bounce looks like a hitch on a
// webview that renders at a lower frame rate, so open is nearly critically damped instead.
export const OPEN: SpringCfg = { response: 0.38, damping: 0.92 };
// Same period as opening, so collapsing is not quicker than expanding (0.32 felt like a jolt now that the open
// pill is big). Critically damped: no rebound at the bottom.
export const CLOSE: SpringCfg = { response: 0.38, damping: 1 };

export class Spring {
  v = 0;
  target: number;
  constructor(public x: number) {
    this.target = x;
  }

  step(dt: number, { response, damping }: SpringCfg) {
    const w = (2 * Math.PI) / response;
    const k = w * w;
    const c = 2 * damping * w;
    const n = Math.max(1, Math.ceil(dt / 0.004));
    const h = dt / n;
    for (let i = 0; i < n; i++) {
      this.v += (-k * (this.x - this.target) - c * this.v) * h;
      this.x += this.v * h;
    }
  }

  get settled() {
    return Math.abs(this.x - this.target) < 0.3 && Math.abs(this.v) < 3;
  }

  snap() {
    this.x = this.target;
    this.v = 0;
  }
}
