import { Directive, inject, OnDestroy, OnInit, TemplateRef } from '@angular/core';
import { HeaderService } from './header.service';

@Directive({
    selector: '[appHeaderTemplate]',
    standalone: true
})
export class HeaderTemplateDirective implements OnInit, OnDestroy {
    private readonly _headerService = inject(HeaderService);
    private readonly _templateRef = inject(TemplateRef<void>);

    public ngOnInit() {
        this._headerService.setTemplate(this._templateRef);
    }

    public ngOnDestroy() {
        this._headerService.clearTemplate();
    }
}
