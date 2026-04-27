import {Component} from '@angular/core';
import {MatButton} from "@angular/material/button";
import {MatDrawer, MatDrawerContainer, MatDrawerContent} from "@angular/material/sidenav";
import {RouterLink, RouterLinkActive} from "@angular/router";
import {CustomRoutes, routes} from "../../../app/app.routes";

import {LogoComponent} from "../logo/logo";
import {ActiveDownloadsComponent} from "../active-downloads/active-downloads";

@Component({
    selector: 'app-sidebar',
    imports: [
        MatButton,
        MatDrawer,
        MatDrawerContainer,
        MatDrawerContent,
        RouterLinkActive,
        RouterLink,
        LogoComponent,
        ActiveDownloadsComponent
    ],
    templateUrl: './sidebar.html',
    styleUrl: './sidebar.scss'
})
export class Sidebar {
    protected readonly routers: CustomRoutes = routes
}
