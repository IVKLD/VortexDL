import { Directive, inject, input, output } from '@angular/core';
import { MusicTrack } from '@shared/models/music-track.model';
import { MusicTracksViewState } from '../music-tracks-view.state';
import { DragSelectService } from './drag-select.service';

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
    private readonly _dragService = inject(DragSelectService);
    private readonly _state = inject(MusicTracksViewState);

    public readonly track = input.required<MusicTrack>({ alias: 'appDragSelect' });
    public readonly itemClick = output<MusicTrack>();

    private applyAction(): void {
        if (this._dragService.dragAction === 'deselect') {
            this._state.deselectTrack(this.track());
        } else {
            this._state.selectTrack(this.track());
        }
    }

    protected onMouseDown(event: MouseEvent): void {
        if (event.button !== 0) return;

        const isSelected = this._state.selectedIds().has(this.track().id);
        this._dragService.startPress(event.clientX, event.clientY, isSelected ? 'deselect' : 'select');

        if (this._state.hasSelection()) {
            this._dragService.startDrag();
            this.applyAction();
        } else {
            this._dragService.longPressTimer = setTimeout(() => {
                if (this._dragService.isMouseDown()) {
                    this._dragService.startDrag();
                    this.applyAction();
                }
            }, 250);
        }
    }

    protected onMouseEnter(): void {
        if (this._dragService.isMouseDown()) {
            this._dragService.startDrag();
            this.applyAction();
        }
    }

    protected onMouseMove(event: MouseEvent): void {
        if (
            this._dragService.isMouseDown() &&
            !this._dragService.isDragActive() &&
            !this._state.hasSelection()
        ) {
            const dx = Math.abs(event.clientX - this._dragService.pressStartX);
            const dy = Math.abs(event.clientY - this._dragService.pressStartY);
            if (dx > 8 || dy > 8) {
                this._dragService.clearTimer();
            }
        }
    }

    protected onWindowMouseUp(): void {
        this._dragService.endDrag();
    }

    protected onClick(event: MouseEvent): void {
        if (this._dragService.justFinishedDrag) {
            event.preventDefault();
            event.stopPropagation();
            this._dragService.justFinishedDrag = false;
            return;
        }

        if (this._state.hasSelection()) {
            this._state.toggleSelect(this.track());
        } else {
            this.itemClick.emit(this.track());
        }
    }
}
