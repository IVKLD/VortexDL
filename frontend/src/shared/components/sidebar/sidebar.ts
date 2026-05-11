import {Component} from '@angular/core';
import {MatButton} from '@angular/material/button';
import {MatSidenav, MatSidenavContainer, MatSidenavContent} from '@angular/material/sidenav';
import {RouterLink, RouterLinkActive} from '@angular/router';
import {LogoComponent} from '../logo/logo';
import {ActiveDownloadsComponent} from '../active-downloads/active-downloads';
import {MatIcon} from '@angular/material/icon';
import {SIDEBAR_NAV_ITEMS} from './sidebar.navigation';
import {Header} from "@shared/components/header/header";

@Component({
    selector: 'app-sidebar',
    imports: [
        RouterLinkActive,
        RouterLink,
        LogoComponent,
        ActiveDownloadsComponent,
        MatIcon,
        MatButton,
        MatSidenavContainer,
        MatSidenav,
        MatSidenavContent,
        Header,
    ],
    templateUrl: './sidebar.html',
    styleUrl: './sidebar.scss',
})
export class Sidebar {
    protected readonly navItems = SIDEBAR_NAV_ITEMS;
}
