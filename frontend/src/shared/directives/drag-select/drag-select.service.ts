import { Injectable, signal } from '@angular/core';
import { fromEvent } from 'rxjs';

export type DragAction = 'select' | 'deselect';

@Injectable({ providedIn: 'root' })
export class DragSelectService {
    public readonly isDragActive = signal(false);

    public dragAction: DragAction = 'select';
    public justFinishedDrag = false;

    constructor() {
        fromEvent(window, 'mouseup').subscribe(() => {
            if (this.isDragActive()) {
                this.endDrag();
            }
        });
    }

    public startDrag(action: DragAction): void {
        this.isDragActive.set(true);
        this.dragAction = action;
        this.justFinishedDrag = true;
    }

    public endDrag(): void {
        this.isDragActive.set(false);
    }
}
