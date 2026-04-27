import {
  CdkDialogContainer,
  Dialog,
  DialogConfig,
  DialogModule
} from "./chunk-4ZPPQB5O.js";
import "./chunk-YOQEV5VJ.js";
import {
  CdkPortalOutlet,
  OverlayModule,
  PortalModule,
  createBlockScrollStrategy,
  createGlobalPositionStrategy
} from "./chunk-5SMZQG26.js";
import {
  CdkScrollable
} from "./chunk-VPWDAHML.js";
import "./chunk-FPU7ERWR.js";
import "./chunk-Y7GC5MM5.js";
import {
  _animationsDisabled
} from "./chunk-IIJJFF3Z.js";
import {
  ESCAPE,
  _IdGenerator,
  coerceNumberProperty,
  hasModifierKey
} from "./chunk-SLLHH7YY.js";
import "./chunk-WV27AK5N.js";
import "./chunk-YRLINXY4.js";
import "./chunk-TEM7VC3E.js";
import "./chunk-VL2OMUKP.js";
import "./chunk-H3ENBYLX.js";
import {
  BidiModule
} from "./chunk-65IB533U.js";
import {
  ChangeDetectionStrategy,
  Component,
  Directive,
  ElementRef,
  EventEmitter,
  Injectable,
  InjectionToken,
  Injector,
  Input,
  NgModule,
  ReplaySubject,
  Subject,
  ViewEncapsulation,
  __spreadProps,
  __spreadValues,
  defer,
  filter,
  inject,
  merge,
  setClassMetadata,
  startWith,
  take,
  ɵɵHostDirectivesFeature,
  ɵɵInheritDefinitionFeature,
  ɵɵNgOnChangesFeature,
  ɵɵattribute,
  ɵɵclassProp,
  ɵɵdefineComponent,
  ɵɵdefineDirective,
  ɵɵdefineInjectable,
  ɵɵdefineInjector,
  ɵɵdefineNgModule,
  ɵɵdomProperty,
  ɵɵelementEnd,
  ɵɵelementStart,
  ɵɵgetInheritedFactory,
  ɵɵlistener,
  ɵɵtemplate
} from "./chunk-ENDTB66Y.js";

// node_modules/@angular/material/fesm2022/dialog.mjs
function MatDialogContainer_ng_template_2_Template(rf, ctx) {
}
var MatDialogConfig = class {
  viewContainerRef;
  injector;
  id;
  role = "dialog";
  panelClass = "";
  hasBackdrop = true;
  backdropClass = "";
  disableClose = false;
  closePredicate;
  width = "";
  height = "";
  minWidth;
  minHeight;
  maxWidth;
  maxHeight;
  position;
  data = null;
  direction;
  ariaDescribedBy = null;
  ariaLabelledBy = null;
  ariaLabel = null;
  ariaModal = false;
  autoFocus = "first-tabbable";
  restoreFocus = true;
  delayFocusTrap = true;
  scrollStrategy;
  closeOnNavigation = true;
  enterAnimationDuration;
  exitAnimationDuration;
};
var OPEN_CLASS = "mdc-dialog--open";
var OPENING_CLASS = "mdc-dialog--opening";
var CLOSING_CLASS = "mdc-dialog--closing";
var OPEN_ANIMATION_DURATION = 150;
var CLOSE_ANIMATION_DURATION = 75;
var MatDialogContainer = class _MatDialogContainer extends CdkDialogContainer {
  _animationStateChanged = new EventEmitter();
  _animationsEnabled = !_animationsDisabled();
  _actionSectionCount = 0;
  _hostElement = this._elementRef.nativeElement;
  _enterAnimationDuration = this._animationsEnabled ? parseCssTime(this._config.enterAnimationDuration) ?? OPEN_ANIMATION_DURATION : 0;
  _exitAnimationDuration = this._animationsEnabled ? parseCssTime(this._config.exitAnimationDuration) ?? CLOSE_ANIMATION_DURATION : 0;
  _animationTimer = null;
  _contentAttached() {
    super._contentAttached();
    this._startOpenAnimation();
  }
  _startOpenAnimation() {
    this._animationStateChanged.emit({
      state: "opening",
      totalTime: this._enterAnimationDuration
    });
    if (this._animationsEnabled) {
      this._hostElement.style.setProperty(TRANSITION_DURATION_PROPERTY, `${this._enterAnimationDuration}ms`);
      this._requestAnimationFrame(() => this._hostElement.classList.add(OPENING_CLASS, OPEN_CLASS));
      this._waitForAnimationToComplete(this._enterAnimationDuration, this._finishDialogOpen);
    } else {
      this._hostElement.classList.add(OPEN_CLASS);
      Promise.resolve().then(() => this._finishDialogOpen());
    }
  }
  _startExitAnimation() {
    this._animationStateChanged.emit({
      state: "closing",
      totalTime: this._exitAnimationDuration
    });
    this._hostElement.classList.remove(OPEN_CLASS);
    if (this._animationsEnabled) {
      this._hostElement.style.setProperty(TRANSITION_DURATION_PROPERTY, `${this._exitAnimationDuration}ms`);
      this._requestAnimationFrame(() => this._hostElement.classList.add(CLOSING_CLASS));
      this._waitForAnimationToComplete(this._exitAnimationDuration, this._finishDialogClose);
    } else {
      Promise.resolve().then(() => this._finishDialogClose());
    }
  }
  _updateActionSectionCount(delta) {
    this._actionSectionCount += delta;
    this._changeDetectorRef.markForCheck();
  }
  _finishDialogOpen = () => {
    this._clearAnimationClasses();
    this._openAnimationDone(this._enterAnimationDuration);
  };
  _finishDialogClose = () => {
    this._clearAnimationClasses();
    this._animationStateChanged.emit({
      state: "closed",
      totalTime: this._exitAnimationDuration
    });
  };
  _clearAnimationClasses() {
    this._hostElement.classList.remove(OPENING_CLASS, CLOSING_CLASS);
  }
  _waitForAnimationToComplete(duration, callback) {
    if (this._animationTimer !== null) {
      clearTimeout(this._animationTimer);
    }
    this._animationTimer = setTimeout(callback, duration);
  }
  _requestAnimationFrame(callback) {
    this._ngZone.runOutsideAngular(() => {
      if (typeof requestAnimationFrame === "function") {
        requestAnimationFrame(callback);
      } else {
        callback();
      }
    });
  }
  _captureInitialFocus() {
    if (!this._config.delayFocusTrap) {
      this._trapFocus();
    }
  }
  _openAnimationDone(totalTime) {
    if (this._config.delayFocusTrap) {
      this._trapFocus();
    }
    this._animationStateChanged.next({
      state: "opened",
      totalTime
    });
  }
  ngOnDestroy() {
    super.ngOnDestroy();
    if (this._animationTimer !== null) {
      clearTimeout(this._animationTimer);
    }
  }
  attachComponentPortal(portal) {
    const ref = super.attachComponentPortal(portal);
    ref.location.nativeElement.classList.add("mat-mdc-dialog-component-host");
    return ref;
  }
  static ɵfac = /* @__PURE__ */ (() => {
    let ɵMatDialogContainer_BaseFactory;
    return function MatDialogContainer_Factory(__ngFactoryType__) {
      return (ɵMatDialogContainer_BaseFactory || (ɵMatDialogContainer_BaseFactory = ɵɵgetInheritedFactory(_MatDialogContainer)))(__ngFactoryType__ || _MatDialogContainer);
    };
  })();
  static ɵcmp = ɵɵdefineComponent({
    type: _MatDialogContainer,
    selectors: [["mat-dialog-container"]],
    hostAttrs: ["tabindex", "-1", 1, "mat-mdc-dialog-container", "mdc-dialog"],
    hostVars: 10,
    hostBindings: function MatDialogContainer_HostBindings(rf, ctx) {
      if (rf & 2) {
        ɵɵdomProperty("id", ctx._config.id);
        ɵɵattribute("aria-modal", ctx._config.ariaModal)("role", ctx._config.role)("aria-labelledby", ctx._config.ariaLabel ? null : ctx._ariaLabelledByQueue[0])("aria-label", ctx._config.ariaLabel)("aria-describedby", ctx._config.ariaDescribedBy || null);
        ɵɵclassProp("_mat-animation-noopable", !ctx._animationsEnabled)("mat-mdc-dialog-container-with-actions", ctx._actionSectionCount > 0);
      }
    },
    features: [ɵɵInheritDefinitionFeature],
    decls: 3,
    vars: 0,
    consts: [[1, "mat-mdc-dialog-inner-container", "mdc-dialog__container"], [1, "mat-mdc-dialog-surface", "mdc-dialog__surface"], ["cdkPortalOutlet", ""]],
    template: function MatDialogContainer_Template(rf, ctx) {
      if (rf & 1) {
        ɵɵelementStart(0, "div", 0)(1, "div", 1);
        ɵɵtemplate(2, MatDialogContainer_ng_template_2_Template, 0, 0, "ng-template", 2);
        ɵɵelementEnd()();
      }
    },
    dependencies: [CdkPortalOutlet],
    styles: ['.mat-mdc-dialog-container {\n  width: 100%;\n  height: 100%;\n  display: block;\n  box-sizing: border-box;\n  max-height: inherit;\n  min-height: inherit;\n  min-width: inherit;\n  max-width: inherit;\n  outline: 0;\n}\n\n.cdk-overlay-pane.mat-mdc-dialog-panel {\n  max-width: var(--mat-dialog-container-max-width, 560px);\n  min-width: var(--mat-dialog-container-min-width, 280px);\n}\n@media (max-width: 599px) {\n  .cdk-overlay-pane.mat-mdc-dialog-panel {\n    max-width: var(--mat-dialog-container-small-max-width, calc(100vw - 32px));\n  }\n}\n\n.mat-mdc-dialog-inner-container {\n  display: flex;\n  flex-direction: row;\n  align-items: center;\n  justify-content: space-around;\n  box-sizing: border-box;\n  height: 100%;\n  opacity: 0;\n  transition: opacity linear var(--mat-dialog-transition-duration, 0ms);\n  max-height: inherit;\n  min-height: inherit;\n  min-width: inherit;\n  max-width: inherit;\n}\n.mdc-dialog--closing .mat-mdc-dialog-inner-container {\n  transition: opacity 75ms linear;\n  transform: none;\n}\n.mdc-dialog--open .mat-mdc-dialog-inner-container {\n  opacity: 1;\n}\n._mat-animation-noopable .mat-mdc-dialog-inner-container {\n  transition: none;\n}\n\n.mat-mdc-dialog-surface {\n  display: flex;\n  flex-direction: column;\n  flex-grow: 0;\n  flex-shrink: 0;\n  box-sizing: border-box;\n  width: 100%;\n  height: 100%;\n  position: relative;\n  overflow-y: auto;\n  outline: 0;\n  transform: scale(0.8);\n  transition: transform var(--mat-dialog-transition-duration, 0ms) cubic-bezier(0, 0, 0.2, 1);\n  max-height: inherit;\n  min-height: inherit;\n  min-width: inherit;\n  max-width: inherit;\n  box-shadow: var(--mat-dialog-container-elevation-shadow, none);\n  border-radius: var(--mat-dialog-container-shape, var(--mat-sys-corner-extra-large, 4px));\n  background-color: var(--mat-dialog-container-color, var(--mat-sys-surface, white));\n}\n[dir=rtl] .mat-mdc-dialog-surface {\n  text-align: right;\n}\n.mdc-dialog--open .mat-mdc-dialog-surface, .mdc-dialog--closing .mat-mdc-dialog-surface {\n  transform: none;\n}\n._mat-animation-noopable .mat-mdc-dialog-surface {\n  transition: none;\n}\n.mat-mdc-dialog-surface::before {\n  position: absolute;\n  box-sizing: border-box;\n  width: 100%;\n  height: 100%;\n  top: 0;\n  left: 0;\n  border: 2px solid transparent;\n  border-radius: inherit;\n  content: "";\n  pointer-events: none;\n}\n\n.mat-mdc-dialog-title {\n  display: block;\n  position: relative;\n  flex-shrink: 0;\n  box-sizing: border-box;\n  margin: 0 0 1px;\n  padding: var(--mat-dialog-headline-padding, 6px 24px 13px);\n}\n.mat-mdc-dialog-title::before {\n  display: inline-block;\n  width: 0;\n  height: 40px;\n  content: "";\n  vertical-align: 0;\n}\n[dir=rtl] .mat-mdc-dialog-title {\n  text-align: right;\n}\n.mat-mdc-dialog-container .mat-mdc-dialog-title {\n  color: var(--mat-dialog-subhead-color, var(--mat-sys-on-surface, rgba(0, 0, 0, 0.87)));\n  font-family: var(--mat-dialog-subhead-font, var(--mat-sys-headline-small-font, inherit));\n  line-height: var(--mat-dialog-subhead-line-height, var(--mat-sys-headline-small-line-height, 1.5rem));\n  font-size: var(--mat-dialog-subhead-size, var(--mat-sys-headline-small-size, 1rem));\n  font-weight: var(--mat-dialog-subhead-weight, var(--mat-sys-headline-small-weight, 400));\n  letter-spacing: var(--mat-dialog-subhead-tracking, var(--mat-sys-headline-small-tracking, 0.03125em));\n}\n\n.mat-mdc-dialog-content {\n  display: block;\n  flex-grow: 1;\n  box-sizing: border-box;\n  margin: 0;\n  overflow: auto;\n  max-height: 65vh;\n}\n.mat-mdc-dialog-content > :first-child {\n  margin-top: 0;\n}\n.mat-mdc-dialog-content > :last-child {\n  margin-bottom: 0;\n}\n.mat-mdc-dialog-container .mat-mdc-dialog-content {\n  color: var(--mat-dialog-supporting-text-color, var(--mat-sys-on-surface-variant, rgba(0, 0, 0, 0.6)));\n  font-family: var(--mat-dialog-supporting-text-font, var(--mat-sys-body-medium-font, inherit));\n  line-height: var(--mat-dialog-supporting-text-line-height, var(--mat-sys-body-medium-line-height, 1.5rem));\n  font-size: var(--mat-dialog-supporting-text-size, var(--mat-sys-body-medium-size, 1rem));\n  font-weight: var(--mat-dialog-supporting-text-weight, var(--mat-sys-body-medium-weight, 400));\n  letter-spacing: var(--mat-dialog-supporting-text-tracking, var(--mat-sys-body-medium-tracking, 0.03125em));\n}\n.mat-mdc-dialog-container .mat-mdc-dialog-content {\n  padding: var(--mat-dialog-content-padding, 20px 24px);\n}\n.mat-mdc-dialog-container-with-actions .mat-mdc-dialog-content {\n  padding: var(--mat-dialog-with-actions-content-padding, 20px 24px 0);\n}\n.mat-mdc-dialog-container .mat-mdc-dialog-title + .mat-mdc-dialog-content {\n  padding-top: 0;\n}\n\n.mat-mdc-dialog-actions {\n  display: flex;\n  position: relative;\n  flex-shrink: 0;\n  flex-wrap: wrap;\n  align-items: center;\n  box-sizing: border-box;\n  min-height: 52px;\n  margin: 0;\n  border-top: 1px solid transparent;\n  padding: var(--mat-dialog-actions-padding, 16px 24px);\n  justify-content: var(--mat-dialog-actions-alignment, flex-end);\n}\n@media (forced-colors: active) {\n  .mat-mdc-dialog-actions {\n    border-top-color: CanvasText;\n  }\n}\n.mat-mdc-dialog-actions.mat-mdc-dialog-actions-align-start, .mat-mdc-dialog-actions[align=start] {\n  justify-content: start;\n}\n.mat-mdc-dialog-actions.mat-mdc-dialog-actions-align-center, .mat-mdc-dialog-actions[align=center] {\n  justify-content: center;\n}\n.mat-mdc-dialog-actions.mat-mdc-dialog-actions-align-end, .mat-mdc-dialog-actions[align=end] {\n  justify-content: flex-end;\n}\n.mat-mdc-dialog-actions .mat-button-base + .mat-button-base,\n.mat-mdc-dialog-actions .mat-mdc-button-base + .mat-mdc-button-base {\n  margin-left: 8px;\n}\n[dir=rtl] .mat-mdc-dialog-actions .mat-button-base + .mat-button-base,\n[dir=rtl] .mat-mdc-dialog-actions .mat-mdc-button-base + .mat-mdc-button-base {\n  margin-left: 0;\n  margin-right: 8px;\n}\n\n.mat-mdc-dialog-component-host {\n  display: contents;\n}\n'],
    encapsulation: 2
  });
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(MatDialogContainer, [{
    type: Component,
    args: [{
      selector: "mat-dialog-container",
      encapsulation: ViewEncapsulation.None,
      changeDetection: ChangeDetectionStrategy.Default,
      imports: [CdkPortalOutlet],
      host: {
        "class": "mat-mdc-dialog-container mdc-dialog",
        "tabindex": "-1",
        "[attr.aria-modal]": "_config.ariaModal",
        "[id]": "_config.id",
        "[attr.role]": "_config.role",
        "[attr.aria-labelledby]": "_config.ariaLabel ? null : _ariaLabelledByQueue[0]",
        "[attr.aria-label]": "_config.ariaLabel",
        "[attr.aria-describedby]": "_config.ariaDescribedBy || null",
        "[class._mat-animation-noopable]": "!_animationsEnabled",
        "[class.mat-mdc-dialog-container-with-actions]": "_actionSectionCount > 0"
      },
      template: '<div class="mat-mdc-dialog-inner-container mdc-dialog__container">\n  <div class="mat-mdc-dialog-surface mdc-dialog__surface">\n    <ng-template cdkPortalOutlet />\n  </div>\n</div>\n',
      styles: ['.mat-mdc-dialog-container {\n  width: 100%;\n  height: 100%;\n  display: block;\n  box-sizing: border-box;\n  max-height: inherit;\n  min-height: inherit;\n  min-width: inherit;\n  max-width: inherit;\n  outline: 0;\n}\n\n.cdk-overlay-pane.mat-mdc-dialog-panel {\n  max-width: var(--mat-dialog-container-max-width, 560px);\n  min-width: var(--mat-dialog-container-min-width, 280px);\n}\n@media (max-width: 599px) {\n  .cdk-overlay-pane.mat-mdc-dialog-panel {\n    max-width: var(--mat-dialog-container-small-max-width, calc(100vw - 32px));\n  }\n}\n\n.mat-mdc-dialog-inner-container {\n  display: flex;\n  flex-direction: row;\n  align-items: center;\n  justify-content: space-around;\n  box-sizing: border-box;\n  height: 100%;\n  opacity: 0;\n  transition: opacity linear var(--mat-dialog-transition-duration, 0ms);\n  max-height: inherit;\n  min-height: inherit;\n  min-width: inherit;\n  max-width: inherit;\n}\n.mdc-dialog--closing .mat-mdc-dialog-inner-container {\n  transition: opacity 75ms linear;\n  transform: none;\n}\n.mdc-dialog--open .mat-mdc-dialog-inner-container {\n  opacity: 1;\n}\n._mat-animation-noopable .mat-mdc-dialog-inner-container {\n  transition: none;\n}\n\n.mat-mdc-dialog-surface {\n  display: flex;\n  flex-direction: column;\n  flex-grow: 0;\n  flex-shrink: 0;\n  box-sizing: border-box;\n  width: 100%;\n  height: 100%;\n  position: relative;\n  overflow-y: auto;\n  outline: 0;\n  transform: scale(0.8);\n  transition: transform var(--mat-dialog-transition-duration, 0ms) cubic-bezier(0, 0, 0.2, 1);\n  max-height: inherit;\n  min-height: inherit;\n  min-width: inherit;\n  max-width: inherit;\n  box-shadow: var(--mat-dialog-container-elevation-shadow, none);\n  border-radius: var(--mat-dialog-container-shape, var(--mat-sys-corner-extra-large, 4px));\n  background-color: var(--mat-dialog-container-color, var(--mat-sys-surface, white));\n}\n[dir=rtl] .mat-mdc-dialog-surface {\n  text-align: right;\n}\n.mdc-dialog--open .mat-mdc-dialog-surface, .mdc-dialog--closing .mat-mdc-dialog-surface {\n  transform: none;\n}\n._mat-animation-noopable .mat-mdc-dialog-surface {\n  transition: none;\n}\n.mat-mdc-dialog-surface::before {\n  position: absolute;\n  box-sizing: border-box;\n  width: 100%;\n  height: 100%;\n  top: 0;\n  left: 0;\n  border: 2px solid transparent;\n  border-radius: inherit;\n  content: "";\n  pointer-events: none;\n}\n\n.mat-mdc-dialog-title {\n  display: block;\n  position: relative;\n  flex-shrink: 0;\n  box-sizing: border-box;\n  margin: 0 0 1px;\n  padding: var(--mat-dialog-headline-padding, 6px 24px 13px);\n}\n.mat-mdc-dialog-title::before {\n  display: inline-block;\n  width: 0;\n  height: 40px;\n  content: "";\n  vertical-align: 0;\n}\n[dir=rtl] .mat-mdc-dialog-title {\n  text-align: right;\n}\n.mat-mdc-dialog-container .mat-mdc-dialog-title {\n  color: var(--mat-dialog-subhead-color, var(--mat-sys-on-surface, rgba(0, 0, 0, 0.87)));\n  font-family: var(--mat-dialog-subhead-font, var(--mat-sys-headline-small-font, inherit));\n  line-height: var(--mat-dialog-subhead-line-height, var(--mat-sys-headline-small-line-height, 1.5rem));\n  font-size: var(--mat-dialog-subhead-size, var(--mat-sys-headline-small-size, 1rem));\n  font-weight: var(--mat-dialog-subhead-weight, var(--mat-sys-headline-small-weight, 400));\n  letter-spacing: var(--mat-dialog-subhead-tracking, var(--mat-sys-headline-small-tracking, 0.03125em));\n}\n\n.mat-mdc-dialog-content {\n  display: block;\n  flex-grow: 1;\n  box-sizing: border-box;\n  margin: 0;\n  overflow: auto;\n  max-height: 65vh;\n}\n.mat-mdc-dialog-content > :first-child {\n  margin-top: 0;\n}\n.mat-mdc-dialog-content > :last-child {\n  margin-bottom: 0;\n}\n.mat-mdc-dialog-container .mat-mdc-dialog-content {\n  color: var(--mat-dialog-supporting-text-color, var(--mat-sys-on-surface-variant, rgba(0, 0, 0, 0.6)));\n  font-family: var(--mat-dialog-supporting-text-font, var(--mat-sys-body-medium-font, inherit));\n  line-height: var(--mat-dialog-supporting-text-line-height, var(--mat-sys-body-medium-line-height, 1.5rem));\n  font-size: var(--mat-dialog-supporting-text-size, var(--mat-sys-body-medium-size, 1rem));\n  font-weight: var(--mat-dialog-supporting-text-weight, var(--mat-sys-body-medium-weight, 400));\n  letter-spacing: var(--mat-dialog-supporting-text-tracking, var(--mat-sys-body-medium-tracking, 0.03125em));\n}\n.mat-mdc-dialog-container .mat-mdc-dialog-content {\n  padding: var(--mat-dialog-content-padding, 20px 24px);\n}\n.mat-mdc-dialog-container-with-actions .mat-mdc-dialog-content {\n  padding: var(--mat-dialog-with-actions-content-padding, 20px 24px 0);\n}\n.mat-mdc-dialog-container .mat-mdc-dialog-title + .mat-mdc-dialog-content {\n  padding-top: 0;\n}\n\n.mat-mdc-dialog-actions {\n  display: flex;\n  position: relative;\n  flex-shrink: 0;\n  flex-wrap: wrap;\n  align-items: center;\n  box-sizing: border-box;\n  min-height: 52px;\n  margin: 0;\n  border-top: 1px solid transparent;\n  padding: var(--mat-dialog-actions-padding, 16px 24px);\n  justify-content: var(--mat-dialog-actions-alignment, flex-end);\n}\n@media (forced-colors: active) {\n  .mat-mdc-dialog-actions {\n    border-top-color: CanvasText;\n  }\n}\n.mat-mdc-dialog-actions.mat-mdc-dialog-actions-align-start, .mat-mdc-dialog-actions[align=start] {\n  justify-content: start;\n}\n.mat-mdc-dialog-actions.mat-mdc-dialog-actions-align-center, .mat-mdc-dialog-actions[align=center] {\n  justify-content: center;\n}\n.mat-mdc-dialog-actions.mat-mdc-dialog-actions-align-end, .mat-mdc-dialog-actions[align=end] {\n  justify-content: flex-end;\n}\n.mat-mdc-dialog-actions .mat-button-base + .mat-button-base,\n.mat-mdc-dialog-actions .mat-mdc-button-base + .mat-mdc-button-base {\n  margin-left: 8px;\n}\n[dir=rtl] .mat-mdc-dialog-actions .mat-button-base + .mat-button-base,\n[dir=rtl] .mat-mdc-dialog-actions .mat-mdc-button-base + .mat-mdc-button-base {\n  margin-left: 0;\n  margin-right: 8px;\n}\n\n.mat-mdc-dialog-component-host {\n  display: contents;\n}\n']
    }]
  }], null, null);
})();
var TRANSITION_DURATION_PROPERTY = "--mat-dialog-transition-duration";
function parseCssTime(time) {
  if (time == null) {
    return null;
  }
  if (typeof time === "number") {
    return time;
  }
  if (time.endsWith("ms")) {
    return coerceNumberProperty(time.substring(0, time.length - 2));
  }
  if (time.endsWith("s")) {
    return coerceNumberProperty(time.substring(0, time.length - 1)) * 1e3;
  }
  if (time === "0") {
    return 0;
  }
  return null;
}
var MatDialogState;
(function(MatDialogState2) {
  MatDialogState2[MatDialogState2["OPEN"] = 0] = "OPEN";
  MatDialogState2[MatDialogState2["CLOSING"] = 1] = "CLOSING";
  MatDialogState2[MatDialogState2["CLOSED"] = 2] = "CLOSED";
})(MatDialogState || (MatDialogState = {}));
var MatDialogRef = class {
  _ref;
  _config;
  _containerInstance;
  componentInstance;
  componentRef = null;
  disableClose;
  id;
  _afterOpened = new ReplaySubject(1);
  _beforeClosed = new ReplaySubject(1);
  _result;
  _closeFallbackTimeout;
  _state = MatDialogState.OPEN;
  _closeInteractionType;
  constructor(_ref, _config, _containerInstance) {
    this._ref = _ref;
    this._config = _config;
    this._containerInstance = _containerInstance;
    this.disableClose = _config.disableClose;
    this.id = _ref.id;
    _ref.addPanelClass("mat-mdc-dialog-panel");
    _containerInstance._animationStateChanged.pipe(filter((event) => event.state === "opened"), take(1)).subscribe(() => {
      this._afterOpened.next();
      this._afterOpened.complete();
    });
    _containerInstance._animationStateChanged.pipe(filter((event) => event.state === "closed"), take(1)).subscribe(() => {
      clearTimeout(this._closeFallbackTimeout);
      this._finishDialogClose();
    });
    _ref.overlayRef.detachments().subscribe(() => {
      this._beforeClosed.next(this._result);
      this._beforeClosed.complete();
      this._finishDialogClose();
    });
    merge(this.backdropClick(), this.keydownEvents().pipe(filter((event) => event.keyCode === ESCAPE && !this.disableClose && !hasModifierKey(event)))).subscribe((event) => {
      if (!this.disableClose) {
        event.preventDefault();
        _closeDialogVia(this, event.type === "keydown" ? "keyboard" : "mouse");
      }
    });
  }
  close(dialogResult) {
    const closePredicate = this._config.closePredicate;
    if (closePredicate && !closePredicate(dialogResult, this._config, this.componentInstance)) {
      return;
    }
    this._result = dialogResult;
    this._containerInstance._animationStateChanged.pipe(filter((event) => event.state === "closing"), take(1)).subscribe((event) => {
      this._beforeClosed.next(dialogResult);
      this._beforeClosed.complete();
      this._ref.overlayRef.detachBackdrop();
      this._closeFallbackTimeout = setTimeout(() => this._finishDialogClose(), event.totalTime + 100);
    });
    this._state = MatDialogState.CLOSING;
    this._containerInstance._startExitAnimation();
  }
  afterOpened() {
    return this._afterOpened;
  }
  afterClosed() {
    return this._ref.closed;
  }
  beforeClosed() {
    return this._beforeClosed;
  }
  backdropClick() {
    return this._ref.backdropClick;
  }
  keydownEvents() {
    return this._ref.keydownEvents;
  }
  updatePosition(position) {
    let strategy = this._ref.config.positionStrategy;
    if (position && (position.left || position.right)) {
      position.left ? strategy.left(position.left) : strategy.right(position.right);
    } else {
      strategy.centerHorizontally();
    }
    if (position && (position.top || position.bottom)) {
      position.top ? strategy.top(position.top) : strategy.bottom(position.bottom);
    } else {
      strategy.centerVertically();
    }
    this._ref.updatePosition();
    return this;
  }
  updateSize(width = "", height = "") {
    this._ref.updateSize(width, height);
    return this;
  }
  addPanelClass(classes) {
    this._ref.addPanelClass(classes);
    return this;
  }
  removePanelClass(classes) {
    this._ref.removePanelClass(classes);
    return this;
  }
  getState() {
    return this._state;
  }
  _finishDialogClose() {
    this._state = MatDialogState.CLOSED;
    this._ref.close(this._result, {
      focusOrigin: this._closeInteractionType
    });
    this.componentInstance = null;
  }
};
function _closeDialogVia(ref, interactionType, result) {
  ref._closeInteractionType = interactionType;
  return ref.close(result);
}
var MAT_DIALOG_DATA = new InjectionToken("MatMdcDialogData");
var MAT_DIALOG_DEFAULT_OPTIONS = new InjectionToken("mat-mdc-dialog-default-options");
var MAT_DIALOG_SCROLL_STRATEGY = new InjectionToken("mat-mdc-dialog-scroll-strategy", {
  providedIn: "root",
  factory: () => {
    const injector = inject(Injector);
    return () => createBlockScrollStrategy(injector);
  }
});
var MatDialog = class _MatDialog {
  _defaultOptions = inject(MAT_DIALOG_DEFAULT_OPTIONS, {
    optional: true
  });
  _scrollStrategy = inject(MAT_DIALOG_SCROLL_STRATEGY);
  _parentDialog = inject(_MatDialog, {
    optional: true,
    skipSelf: true
  });
  _idGenerator = inject(_IdGenerator);
  _injector = inject(Injector);
  _dialog = inject(Dialog);
  _animationsDisabled = _animationsDisabled();
  _openDialogsAtThisLevel = [];
  _afterAllClosedAtThisLevel = new Subject();
  _afterOpenedAtThisLevel = new Subject();
  dialogConfigClass = MatDialogConfig;
  _dialogRefConstructor;
  _dialogContainerType;
  _dialogDataToken;
  get openDialogs() {
    return this._parentDialog ? this._parentDialog.openDialogs : this._openDialogsAtThisLevel;
  }
  get afterOpened() {
    return this._parentDialog ? this._parentDialog.afterOpened : this._afterOpenedAtThisLevel;
  }
  _getAfterAllClosed() {
    const parent = this._parentDialog;
    return parent ? parent._getAfterAllClosed() : this._afterAllClosedAtThisLevel;
  }
  afterAllClosed = defer(() => this.openDialogs.length ? this._getAfterAllClosed() : this._getAfterAllClosed().pipe(startWith(void 0)));
  constructor() {
    this._dialogRefConstructor = MatDialogRef;
    this._dialogContainerType = MatDialogContainer;
    this._dialogDataToken = MAT_DIALOG_DATA;
  }
  open(componentOrTemplateRef, config) {
    let dialogRef;
    config = __spreadValues(__spreadValues({}, this._defaultOptions || new MatDialogConfig()), config);
    config.id = config.id || this._idGenerator.getId("mat-mdc-dialog-");
    config.scrollStrategy = config.scrollStrategy || this._scrollStrategy();
    const cdkRef = this._dialog.open(componentOrTemplateRef, __spreadProps(__spreadValues({}, config), {
      positionStrategy: createGlobalPositionStrategy(this._injector).centerHorizontally().centerVertically(),
      disableClose: true,
      closePredicate: void 0,
      closeOnDestroy: false,
      closeOnOverlayDetachments: false,
      disableAnimations: this._animationsDisabled || config.enterAnimationDuration?.toLocaleString() === "0" || config.exitAnimationDuration?.toString() === "0",
      container: {
        type: this._dialogContainerType,
        providers: () => [{
          provide: this.dialogConfigClass,
          useValue: config
        }, {
          provide: DialogConfig,
          useValue: config
        }]
      },
      templateContext: () => ({
        dialogRef
      }),
      providers: (ref, cdkConfig, dialogContainer) => {
        dialogRef = new this._dialogRefConstructor(ref, config, dialogContainer);
        dialogRef.updatePosition(config?.position);
        return [{
          provide: this._dialogContainerType,
          useValue: dialogContainer
        }, {
          provide: this._dialogDataToken,
          useValue: cdkConfig.data
        }, {
          provide: this._dialogRefConstructor,
          useValue: dialogRef
        }];
      }
    }));
    dialogRef.componentRef = cdkRef.componentRef;
    dialogRef.componentInstance = cdkRef.componentInstance;
    this.openDialogs.push(dialogRef);
    this.afterOpened.next(dialogRef);
    dialogRef.afterClosed().subscribe(() => {
      const index = this.openDialogs.indexOf(dialogRef);
      if (index > -1) {
        this.openDialogs.splice(index, 1);
        if (!this.openDialogs.length) {
          this._getAfterAllClosed().next();
        }
      }
    });
    return dialogRef;
  }
  closeAll() {
    this._closeDialogs(this.openDialogs);
  }
  getDialogById(id) {
    return this.openDialogs.find((dialog) => dialog.id === id);
  }
  ngOnDestroy() {
    this._closeDialogs(this._openDialogsAtThisLevel);
    this._afterAllClosedAtThisLevel.complete();
    this._afterOpenedAtThisLevel.complete();
  }
  _closeDialogs(dialogs) {
    let i = dialogs.length;
    while (i--) {
      dialogs[i].close();
    }
  }
  static ɵfac = function MatDialog_Factory(__ngFactoryType__) {
    return new (__ngFactoryType__ || _MatDialog)();
  };
  static ɵprov = ɵɵdefineInjectable({
    token: _MatDialog,
    factory: _MatDialog.ɵfac,
    providedIn: "root"
  });
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(MatDialog, [{
    type: Injectable,
    args: [{
      providedIn: "root"
    }]
  }], () => [], null);
})();
var MatDialogClose = class _MatDialogClose {
  dialogRef = inject(MatDialogRef, {
    optional: true
  });
  _elementRef = inject(ElementRef);
  _dialog = inject(MatDialog);
  ariaLabel;
  type = "button";
  dialogResult;
  _matDialogClose;
  constructor() {
  }
  ngOnInit() {
    if (!this.dialogRef) {
      this.dialogRef = getClosestDialog(this._elementRef, this._dialog.openDialogs);
    }
  }
  ngOnChanges(changes) {
    const proxiedChange = changes["_matDialogClose"] || changes["_matDialogCloseResult"];
    if (proxiedChange) {
      this.dialogResult = proxiedChange.currentValue;
    }
  }
  _onButtonClick(event) {
    _closeDialogVia(this.dialogRef, event.screenX === 0 && event.screenY === 0 ? "keyboard" : "mouse", this.dialogResult);
  }
  static ɵfac = function MatDialogClose_Factory(__ngFactoryType__) {
    return new (__ngFactoryType__ || _MatDialogClose)();
  };
  static ɵdir = ɵɵdefineDirective({
    type: _MatDialogClose,
    selectors: [["", "mat-dialog-close", ""], ["", "matDialogClose", ""]],
    hostVars: 2,
    hostBindings: function MatDialogClose_HostBindings(rf, ctx) {
      if (rf & 1) {
        ɵɵlistener("click", function MatDialogClose_click_HostBindingHandler($event) {
          return ctx._onButtonClick($event);
        });
      }
      if (rf & 2) {
        ɵɵattribute("aria-label", ctx.ariaLabel || null)("type", ctx.type);
      }
    },
    inputs: {
      ariaLabel: [0, "aria-label", "ariaLabel"],
      type: "type",
      dialogResult: [0, "mat-dialog-close", "dialogResult"],
      _matDialogClose: [0, "matDialogClose", "_matDialogClose"]
    },
    exportAs: ["matDialogClose"],
    features: [ɵɵNgOnChangesFeature]
  });
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(MatDialogClose, [{
    type: Directive,
    args: [{
      selector: "[mat-dialog-close], [matDialogClose]",
      exportAs: "matDialogClose",
      host: {
        "(click)": "_onButtonClick($event)",
        "[attr.aria-label]": "ariaLabel || null",
        "[attr.type]": "type"
      }
    }]
  }], () => [], {
    ariaLabel: [{
      type: Input,
      args: ["aria-label"]
    }],
    type: [{
      type: Input
    }],
    dialogResult: [{
      type: Input,
      args: ["mat-dialog-close"]
    }],
    _matDialogClose: [{
      type: Input,
      args: ["matDialogClose"]
    }]
  });
})();
var MatDialogLayoutSection = class _MatDialogLayoutSection {
  _dialogRef = inject(MatDialogRef, {
    optional: true
  });
  _elementRef = inject(ElementRef);
  _dialog = inject(MatDialog);
  constructor() {
  }
  ngOnInit() {
    if (!this._dialogRef) {
      this._dialogRef = getClosestDialog(this._elementRef, this._dialog.openDialogs);
    }
    if (this._dialogRef) {
      Promise.resolve().then(() => {
        this._onAdd();
      });
    }
  }
  ngOnDestroy() {
    const instance = this._dialogRef?._containerInstance;
    if (instance) {
      Promise.resolve().then(() => {
        this._onRemove();
      });
    }
  }
  static ɵfac = function MatDialogLayoutSection_Factory(__ngFactoryType__) {
    return new (__ngFactoryType__ || _MatDialogLayoutSection)();
  };
  static ɵdir = ɵɵdefineDirective({
    type: _MatDialogLayoutSection
  });
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(MatDialogLayoutSection, [{
    type: Directive
  }], () => [], null);
})();
var MatDialogTitle = class _MatDialogTitle extends MatDialogLayoutSection {
  id = inject(_IdGenerator).getId("mat-mdc-dialog-title-");
  _onAdd() {
    this._dialogRef._containerInstance?._addAriaLabelledBy?.(this.id);
  }
  _onRemove() {
    this._dialogRef?._containerInstance?._removeAriaLabelledBy?.(this.id);
  }
  static ɵfac = /* @__PURE__ */ (() => {
    let ɵMatDialogTitle_BaseFactory;
    return function MatDialogTitle_Factory(__ngFactoryType__) {
      return (ɵMatDialogTitle_BaseFactory || (ɵMatDialogTitle_BaseFactory = ɵɵgetInheritedFactory(_MatDialogTitle)))(__ngFactoryType__ || _MatDialogTitle);
    };
  })();
  static ɵdir = ɵɵdefineDirective({
    type: _MatDialogTitle,
    selectors: [["", "mat-dialog-title", ""], ["", "matDialogTitle", ""]],
    hostAttrs: [1, "mat-mdc-dialog-title", "mdc-dialog__title"],
    hostVars: 1,
    hostBindings: function MatDialogTitle_HostBindings(rf, ctx) {
      if (rf & 2) {
        ɵɵdomProperty("id", ctx.id);
      }
    },
    inputs: {
      id: "id"
    },
    exportAs: ["matDialogTitle"],
    features: [ɵɵInheritDefinitionFeature]
  });
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(MatDialogTitle, [{
    type: Directive,
    args: [{
      selector: "[mat-dialog-title], [matDialogTitle]",
      exportAs: "matDialogTitle",
      host: {
        "class": "mat-mdc-dialog-title mdc-dialog__title",
        "[id]": "id"
      }
    }]
  }], null, {
    id: [{
      type: Input
    }]
  });
})();
var MatDialogContent = class _MatDialogContent {
  static ɵfac = function MatDialogContent_Factory(__ngFactoryType__) {
    return new (__ngFactoryType__ || _MatDialogContent)();
  };
  static ɵdir = ɵɵdefineDirective({
    type: _MatDialogContent,
    selectors: [["", "mat-dialog-content", ""], ["mat-dialog-content"], ["", "matDialogContent", ""]],
    hostAttrs: [1, "mat-mdc-dialog-content", "mdc-dialog__content"],
    features: [ɵɵHostDirectivesFeature([CdkScrollable])]
  });
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(MatDialogContent, [{
    type: Directive,
    args: [{
      selector: `[mat-dialog-content], mat-dialog-content, [matDialogContent]`,
      host: {
        "class": "mat-mdc-dialog-content mdc-dialog__content"
      },
      hostDirectives: [CdkScrollable]
    }]
  }], null, null);
})();
var MatDialogActions = class _MatDialogActions extends MatDialogLayoutSection {
  align;
  _onAdd() {
    this._dialogRef._containerInstance?._updateActionSectionCount?.(1);
  }
  _onRemove() {
    this._dialogRef._containerInstance?._updateActionSectionCount?.(-1);
  }
  static ɵfac = /* @__PURE__ */ (() => {
    let ɵMatDialogActions_BaseFactory;
    return function MatDialogActions_Factory(__ngFactoryType__) {
      return (ɵMatDialogActions_BaseFactory || (ɵMatDialogActions_BaseFactory = ɵɵgetInheritedFactory(_MatDialogActions)))(__ngFactoryType__ || _MatDialogActions);
    };
  })();
  static ɵdir = ɵɵdefineDirective({
    type: _MatDialogActions,
    selectors: [["", "mat-dialog-actions", ""], ["mat-dialog-actions"], ["", "matDialogActions", ""]],
    hostAttrs: [1, "mat-mdc-dialog-actions", "mdc-dialog__actions"],
    hostVars: 6,
    hostBindings: function MatDialogActions_HostBindings(rf, ctx) {
      if (rf & 2) {
        ɵɵclassProp("mat-mdc-dialog-actions-align-start", ctx.align === "start")("mat-mdc-dialog-actions-align-center", ctx.align === "center")("mat-mdc-dialog-actions-align-end", ctx.align === "end");
      }
    },
    inputs: {
      align: "align"
    },
    features: [ɵɵInheritDefinitionFeature]
  });
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(MatDialogActions, [{
    type: Directive,
    args: [{
      selector: `[mat-dialog-actions], mat-dialog-actions, [matDialogActions]`,
      host: {
        "class": "mat-mdc-dialog-actions mdc-dialog__actions",
        "[class.mat-mdc-dialog-actions-align-start]": 'align === "start"',
        "[class.mat-mdc-dialog-actions-align-center]": 'align === "center"',
        "[class.mat-mdc-dialog-actions-align-end]": 'align === "end"'
      }
    }]
  }], null, {
    align: [{
      type: Input
    }]
  });
})();
function getClosestDialog(element, openDialogs) {
  let parent = element.nativeElement.parentElement;
  while (parent && !parent.classList.contains("mat-mdc-dialog-container")) {
    parent = parent.parentElement;
  }
  return parent ? openDialogs.find((dialog) => dialog.id === parent.id) : null;
}
var DIRECTIVES = [MatDialogContainer, MatDialogClose, MatDialogTitle, MatDialogActions, MatDialogContent];
var MatDialogModule = class _MatDialogModule {
  static ɵfac = function MatDialogModule_Factory(__ngFactoryType__) {
    return new (__ngFactoryType__ || _MatDialogModule)();
  };
  static ɵmod = ɵɵdefineNgModule({
    type: _MatDialogModule,
    imports: [DialogModule, OverlayModule, PortalModule, MatDialogContainer, MatDialogClose, MatDialogTitle, MatDialogActions, MatDialogContent],
    exports: [BidiModule, MatDialogContainer, MatDialogClose, MatDialogTitle, MatDialogActions, MatDialogContent]
  });
  static ɵinj = ɵɵdefineInjector({
    providers: [MatDialog],
    imports: [DialogModule, OverlayModule, PortalModule, BidiModule]
  });
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(MatDialogModule, [{
    type: NgModule,
    args: [{
      imports: [DialogModule, OverlayModule, PortalModule, ...DIRECTIVES],
      exports: [BidiModule, ...DIRECTIVES],
      providers: [MatDialog]
    }]
  }], null, null);
})();
export {
  MAT_DIALOG_DATA,
  MAT_DIALOG_DEFAULT_OPTIONS,
  MAT_DIALOG_SCROLL_STRATEGY,
  MatDialog,
  MatDialogActions,
  MatDialogClose,
  MatDialogConfig,
  MatDialogContainer,
  MatDialogContent,
  MatDialogModule,
  MatDialogRef,
  MatDialogState,
  MatDialogTitle,
  _closeDialogVia
};
//# sourceMappingURL=@angular_material_dialog.js.map
