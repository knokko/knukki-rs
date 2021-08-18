export function compute_metrics(grapheme, font) {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    ctx.font = font;
    const metrics = ctx.measureText(grapheme);
    return new CustomTextMetrics(
        Math.floor(metrics.actualBoundingBoxLeft), Math.ceil(metrics.actualBoundingBoxDescent),
        Math.ceil(metrics.actualBoundingBoxRight), Math.ceil(metrics.actualBoundingBoxAscent)
    );
}

export class CustomTextMetrics {
    constructor(actualLeft, actualDescent, actualRight, actualAscent) {
        this._actualLeft = actualLeft;
        this._actualDescent = actualDescent;
        this._actualRight = actualRight;
        this._actualAscent = actualAscent;
    }

    get actual_left() {
        return this._actualLeft;
    }

    get actual_descent() {
        return this._actualDescent;
    }

    get actual_right() {
        return this._actualRight;
    }

    get actual_ascent() {
        return this._actualAscent;
    }
}