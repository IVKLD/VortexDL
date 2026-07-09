import {Injectable, signal} from '@angular/core';

export enum NotificationType {
    Info = 'info',
    Success = 'success',
    Error = 'error'
}

export interface AppNotification {
    id: string;
    message: string;
    type: NotificationType;
    dismissing?: boolean;
    shaking?: boolean;
}

@Injectable({
    providedIn: 'root'
})
export class NotificationService {
    private readonly _notifications = signal<AppNotification[]>([]);
    private readonly _timeouts = new Map<string, ReturnType<typeof setTimeout>>();
    private readonly MAX_NOTIFICATIONS = 5;

    public readonly notifications = this._notifications.asReadonly();

    private show(message: string, type: NotificationType, duration = 4000) {
        // De-duplication: If same active notification exists, refresh its timeout and trigger shake
        const existing = this._notifications().find(
            n => n.message === message && n.type === type && !n.dismissing
        );

        if (existing) {
            const existingTimeout = this._timeouts.get(existing.id);
            if (existingTimeout) {
                clearTimeout(existingTimeout);
            }

            // Trigger shake
            this._notifications.update(prev =>
                prev.map(n => n.id === existing.id ? { ...n, shaking: true } : n)
            );

            // Stop shake animation after 500ms
            setTimeout(() => {
                this._notifications.update(prev =>
                    prev.map(n => n.id === existing.id ? { ...n, shaking: false } : n)
                );
            }, 500);

            if (duration > 0) {
                const newTimeout = setTimeout(() => {
                    this.dismiss(existing.id);
                }, duration);
                this._timeouts.set(existing.id, newTimeout);
            }
            return;
        }

        // Limit concurrent notifications
        const activeNotifications = this._notifications().filter(n => !n.dismissing);
        if (activeNotifications.length >= this.MAX_NOTIFICATIONS) {
            const oldest = activeNotifications[0];
            if (oldest) {
                this.dismiss(oldest.id);
            }
        }

        const id = Math.random().toString(36).substring(2, 9);
        const notification: AppNotification = {id, message, type};

        this._notifications.update(prev => [...prev, notification]);

        if (duration > 0) {
            const timeout = setTimeout(() => {
                this.dismiss(id);
            }, duration);
            this._timeouts.set(id, timeout);
        }
    }

    public info(message: string, duration = 4000) {
        this.show(message, NotificationType.Info, duration);
    }

    public success(message: string, duration = 4000) {
        this.show(message, NotificationType.Success, duration);
    }

    public error(message: string, duration = 4000) {
        this.show(message, NotificationType.Error, duration);
    }

    public dismiss(id: string) {
        const timeout = this._timeouts.get(id);
        if (timeout) {
            clearTimeout(timeout);
            this._timeouts.delete(id);
        }

        this._notifications.update(prev =>
            prev.map(n => n.id === id ? { ...n, dismissing: true } : n)
        );

        setTimeout(() => {
            this._notifications.update(prev => prev.filter(n => n.id !== id));
        }, 300);
    }
}
