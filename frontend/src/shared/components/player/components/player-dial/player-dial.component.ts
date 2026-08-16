import {Component, computed, ElementRef, inject, OnDestroy, OnInit, Renderer2, viewChild} from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { PlayerService } from '@app/services/player.service';
import {MatButton} from "@angular/material/button";

import { MatProgressSpinner } from '@angular/material/progress-spinner';

@Component({
    selector: 'app-player-dial',
    imports: [
        MatIcon,
        MatButton,
        MatProgressSpinner,
    ],
    templateUrl: './player-dial.component.html',
    styleUrl: './player-dial.component.scss',
    host: {
        '(mousedown)': 'onMouseDown($event)',
        '(touchstart)': 'onTouchStart($event)',
        '(mouseenter)': 'onMouseEnter()',
        '(mouseleave)': 'onMouseLeave()'
    }
})
export class PlayerDialComponent implements OnInit, OnDestroy {
    private readonly elementRef = inject<ElementRef<HTMLElement>>(ElementRef);
    private readonly renderer = inject(Renderer2);
    protected readonly player = inject(PlayerService);

    private readonly progressCircle = viewChild.required<ElementRef<SVGCircleElement>>('progressCircle');
    private readonly timerText = viewChild.required<ElementRef<HTMLSpanElement>>('timerText');

    private isDragging = false;
    private isHovered = false;
    private dragProgress = 0;
    
    private cachedCenterX = 0;
    private cachedCenterY = 0;

    private dragMoveListener?: () => void;
    private dragUpListener?: () => void;
    
    private animationFrameId?: number;
    private lastText = '';

    private runLoop = (): void => {
        this.updateUI();
        this.animationFrameId = requestAnimationFrame(this.runLoop);
    };

    protected readonly playIcon = computed(() =>
        this.player.isPlaying() ? 'pause' : 'play_arrow'
    );

    private updateUI(): void {
        const duration = this.player.duration() || 0;
        const progress = this.isDragging ? this.dragProgress : this.player.audio.currentTime;

        const ratio = duration > 0 ? Math.max(0, Math.min(1, progress / duration)) : 0;
        const offset = 276.46 * (1 - ratio);

        this.renderer.setStyle(this.progressCircle().nativeElement, 'stroke-dashoffset', `${offset}`);

        if (this.isHovered || this.isDragging) {
            const text = `${this.formatTime(progress)} / ${this.formatTime(duration)}`;
            if (this.lastText !== text) {
                this.lastText = text;
                this.renderer.setProperty(this.timerText().nativeElement, 'innerText', text);
            }
        }
    }

    private startDrag(clientX: number, clientY: number): void {
        this.cleanupDrag();
        this.isDragging = true;
        
        const rect = this.elementRef.nativeElement.getBoundingClientRect();
        this.cachedCenterX = rect.left + rect.width / 2;
        this.cachedCenterY = rect.top + rect.height / 2;

        this.updateDragProgress(clientX, clientY);
    }

    private updateDragProgress(clientX: number, clientY: number): void {
        const duration = this.player.duration();
        if (!duration) return;

        const dx = clientX - this.cachedCenterX;
        const dy = clientY - this.cachedCenterY;

        let angle = Math.atan2(dy, dx) + Math.PI / 2;
        if (angle < 0) {
            angle += 2 * Math.PI;
        }

        const ratio = angle / (2 * Math.PI);
        this.dragProgress = ratio * duration;
    }

    private endDrag(): void {
        if (this.isDragging) {
            this.player.seek(this.dragProgress);
        }
        this.cleanupDrag();
    }

    private cleanupDrag(): void {
        this.isDragging = false;
        if (this.dragMoveListener) {
            this.dragMoveListener();
            this.dragMoveListener = undefined;
        }
        if (this.dragUpListener) {
            this.dragUpListener();
            this.dragUpListener = undefined;
        }
    }

    private formatTime(seconds: number): string {
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }

    protected onMouseEnter(): void {
        this.isHovered = true;
    }

    protected onMouseLeave(): void {
        this.isHovered = false;
    }

    protected onMouseDown(event: MouseEvent): void {
        this.startDrag(event.clientX, event.clientY);

        this.dragMoveListener = this.renderer.listen('document', 'mousemove', (moveEvent: MouseEvent) => {
            this.updateDragProgress(moveEvent.clientX, moveEvent.clientY);
        });

        this.dragUpListener = this.renderer.listen('document', 'mouseup', () => {
            this.endDrag();
        });
    }

    protected onTouchStart(event: TouchEvent): void {
        const touch = event.touches[0];
        if (!touch) return;
        
        this.startDrag(touch.clientX, touch.clientY);

        this.dragMoveListener = this.renderer.listen('document', 'touchmove', (moveEvent: TouchEvent) => {
            const moveTouch = moveEvent.touches[0];
            if (moveTouch) {
                this.updateDragProgress(moveTouch.clientX, moveTouch.clientY);
            }
        });

        this.dragUpListener = this.renderer.listen('document', 'touchend', () => {
            this.endDrag();
        });
    }

    ngOnInit(): void {
        this.runLoop();
    }

    ngOnDestroy(): void {
        this.cleanupDrag();
        if (this.animationFrameId) {
            cancelAnimationFrame(this.animationFrameId);
        }
    }
}
