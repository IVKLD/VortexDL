import {Component, inject} from '@angular/core';
import {ActivatedRoute, NavigationEnd, Router} from '@angular/router';
import {HeaderConfig} from './header.types';
import {filter, map} from 'rxjs';
import {toSignal} from '@angular/core/rxjs-interop';
import {RouteData} from '@app/app.routes';
import {HeaderLeft} from './components/header-left/header-left.component';
import {HeaderCenter} from './components/header-center/header-center.component';
import {HeaderRight} from './components/header-right/header-right.component';

@Component({
    selector: 'app-header',
    imports: [HeaderLeft, HeaderCenter, HeaderRight],
    templateUrl: './header.html',
    styleUrl: './header.scss',
    })
export class Header {
    private readonly _router = inject(Router);
    private readonly _route = inject(ActivatedRoute);

    private readonly _headerConfig$ = this._router.events.pipe(
        filter(event => event instanceof NavigationEnd),
        map(() => {
            let route = this._route.root;
            while (route.firstChild) {
                route = route.firstChild;
            }
            const data: RouteData = route.snapshot.data;
            return data.header;
        }),
    );
    protected readonly config = toSignal(this._headerConfig$, {
        initialValue: this.getInitialHeaderConfig(),
    });

    private getInitialHeaderConfig(): HeaderConfig | undefined {
        const data: RouteData = this._route.root.snapshot.firstChild?.data || {};
        return data.header;
    }
}

