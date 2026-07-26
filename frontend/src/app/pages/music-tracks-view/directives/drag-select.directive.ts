import { Directive, inject, input, output } from '@angular/core';
import { MusicTrack } from '@shared/models/music-track.model';
import { MusicTracksViewState } from '../music-tracks-view.state';

@Directive({
    selector: '[appDragSelect]',
    host: {
        '(mousedown)': 'onMouseDown($event)',
        '(mouseenter)': 'onMouseEnter()',
        '(mousemove)': 'onMouseMove($event)',
        '(click)': 'onClick($event)',
        '(window:mouseup)': 'onWindowMouseUp()',
    },
})
export class DragSelectDirective {
    private static isMouseDown = false;
    private static isDragActive = false;
    private static dragAction: 'select' | 'deselect' = 'select';
    private static longPressTimer?: ReturnType<typeof setTimeout>;
    private static pressStartX = 0;
    private static pressStartY = 0;
    private static justFinishedDrag = false;

    private readonly _state = inject(MusicTracksViewState);

    public readonly track = input.required<MusicTrack>({ alias: 'appDragSelect' });
    public readonly itemClick = output<MusicTrack>();

    private applyAction(): void {
        if (DragSelectDirective.dragAction === 'deselect') {
            this._state.deselectTrack(this.track());
        } else {
            this._state.selectTrack(this.track());
        }
    }

    protected onMouseDown(event: MouseEvent): void {
        if (event.button !== 0) return;

        const isSelected = this._state.selectedIds().has(this.track().id);
        DragSelectDirective.isMouseDown = true;
        DragSelectDirective.isDragActive = false;
        DragSelectDirective.dragAction = isSelected ? 'deselect' : 'select';
        DragSelectDirective.pressStartX = event.clientX;
        DragSelectDirective.pressStartY = event.clientY;
        DragSelectDirective.justFinishedDrag = false;

        clearTimeout(DragSelectDirective.longPressTimer);

        if (this._state.hasSelection()) {
            DragSelectDirective.isDragActive = true;
            DragSelectDirective.justFinishedDrag = true;
            this.applyAction();
        } else {
            DragSelectDirective.longPressTimer = setTimeout(() => {
                if (DragSelectDirective.isMouseDown) {
                    DragSelectDirective.isDragActive = true;
                    DragSelectDirective.justFinishedDrag = true;
                    this.applyAction();
                }
            }, 250);
        }
    }

    protected onMouseEnter(): void {
        if (DragSelectDirective.isMouseDown) {
            DragSelectDirective.isDragActive = true;
            DragSelectDirective.justFinishedDrag = true;
            this.applyAction();
        }
    }

    protected onMouseMove(event: MouseEvent): void {
        if (
            DragSelectDirective.isMouseDown &&
            !DragSelectDirective.isDragActive &&
            !this._state.hasSelection()
        ) {
            const dx = Math.abs(event.clientX - DragSelectDirective.pressStartX);
            const dy = Math.abs(event.clientY - DragSelectDirective.pressStartY);
            if (dx > 8 || dy > 8) {
                clearTimeout(DragSelectDirective.longPressTimer);
            }
        }
    }

    protected onWindowMouseUp(): void {
        clearTimeout(DragSelectDirective.longPressTimer);
        DragSelectDirective.isMouseDown = false;
        DragSelectDirective.isDragActive = false;
    }

    protected onClick(event: MouseEvent): void {
        if (DragSelectDirective.justFinishedDrag) {
            event.preventDefault();
            event.stopPropagation();
            DragSelectDirective.justFinishedDrag = false;
            return;
        }

        if (this._state.hasSelection()) {
            this._state.toggleSelect(this.track());
        } else {
            this.itemClick.emit(this.track());
        }
    }
}
