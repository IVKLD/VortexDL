import {Component, inject} from '@angular/core';
import {NotificationService} from '@app/services/notification.service';
import {MatIcon} from '@angular/material/icon';

@Component({
    selector: 'app-sidebar-notifications',
    imports: [MatIcon],
    templateUrl: './sidebar-notifications.html',
    styleUrl: './sidebar-notifications.scss',
})
export class SidebarNotifications {
    protected readonly service = inject(NotificationService);
}
