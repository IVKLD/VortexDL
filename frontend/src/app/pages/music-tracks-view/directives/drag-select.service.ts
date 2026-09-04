import { Injectable, signal } from '@angular/core';

export type DragAction = 'select' | 'deselect';

@Injectable({ providedIn: 'root' })
export class DragSelectService {
    public readonly isMouseDown = signal(false);
    public readonly isDragActive = signal(false);

    public dragAction: DragAction = 'select';
    public pressStartX = 0;
    public pressStartY = 0;
    public justFinishedDrag = false;
    public longPressTimer?: ReturnType<typeof setTimeout>;

    public startPress(x: number, y: number, action: DragAction): void {
        this.clearTimer();
        this.isMouseDown.set(true);
        this.isDragActive.set(false);
        this.dragAction = action;
        this.pressStartX = x;
        this.pressStartY = y;
        this.justFinishedDrag = false;
    }

    public startDrag(): void {
        this.clearTimer();
        this.isDragActive.set(true);
        this.justFinishedDrag = true;
    }

    public endDrag(): void {
        this.clearTimer();
        this.isMouseDown.set(false);
        this.isDragActive.set(false);
    }

    public clearTimer(): void {
        if (this.longPressTimer) {
            clearTimeout(this.longPressTimer);
            this.longPressTimer = undefined;
        }
    }
}
