import { Component, contentChildren, TemplateRef } from '@angular/core';
import { NgTemplateOutlet } from '@angular/common';
import { MatDivider } from '@angular/material/divider';

@Component({
    selector: 'app-button-group',
    imports: [NgTemplateOutlet, MatDivider],
    templateUrl: './button-group.html',
    styleUrl: './button-group.scss',
})
export class ButtonGroup<T = void> {
    protected readonly templates = contentChildren<TemplateRef<T>>(TemplateRef);
}
