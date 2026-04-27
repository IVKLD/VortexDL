import {
  BehaviorSubject,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  ContentChild,
  DOCUMENT,
  DestroyRef,
  Directive,
  ElementRef,
  ErrorHandler,
  Inject,
  Injectable,
  InjectionToken,
  Injector,
  Input,
  IterableDiffers,
  NEVER,
  NgZone,
  Observable,
  Optional,
  Output,
  ReplaySubject,
  Subject,
  Subscription,
  TemplateRef,
  ViewChild,
  ViewContainerRef,
  ViewEncapsulation,
  __spreadProps,
  __spreadValues,
  _global,
  assertInInjectionContext,
  catchError,
  combineLatest,
  concat,
  defer,
  distinctUntilChanged,
  effect,
  exhaustMap,
  filter,
  finalize,
  from,
  fromEvent,
  groupBy,
  ignoreElements,
  inject,
  isObservable,
  isSignal,
  map,
  mapTo,
  merge,
  mergeMap,
  of,
  pairwise,
  setClassMetadata,
  share,
  shareReplay,
  startWith,
  switchAll,
  switchMap,
  take,
  takeUntil,
  takeWhile,
  tap,
  throwError,
  untracked,
  ɵɵInheritDefinitionFeature,
  ɵɵNgOnChangesFeature,
  ɵɵProvidersFeature,
  ɵɵadvance,
  ɵɵclassProp,
  ɵɵconditional,
  ɵɵconditionalCreate,
  ɵɵcontentQuery,
  ɵɵdefineComponent,
  ɵɵdefineDirective,
  ɵɵdefineInjectable,
  ɵɵdirectiveInject,
  ɵɵdomElement,
  ɵɵdomElementEnd,
  ɵɵdomElementStart,
  ɵɵgetInheritedFactory,
  ɵɵinject,
  ɵɵloadQuery,
  ɵɵprojection,
  ɵɵprojectionDef,
  ɵɵqueryRefresh,
  ɵɵviewQuery
} from "./chunk-ENDTB66Y.js";

// node_modules/@rx-angular/cdk/fesm2022/rx-angular-cdk-coalescing.mjs
var coalescingManager = createCoalesceManager();
function hasKey(ctx, property) {
  return ctx[property] != null;
}
function createPropertiesWeakMap(getDefaults) {
  const propertyMap = /* @__PURE__ */ new WeakMap();
  return {
    getProps: getProperties,
    setProps: setProperties
  };
  function getProperties(ctx) {
    const defaults = getDefaults(ctx);
    const propertiesPresent = propertyMap.get(ctx);
    let properties;
    if (propertiesPresent !== void 0) {
      properties = propertiesPresent;
    } else {
      properties = {};
      Object.entries(defaults).forEach(([prop, value]) => {
        if (hasKey(ctx, prop)) {
          properties[prop] = ctx[prop];
        } else {
          properties[prop] = value;
        }
      });
      propertyMap.set(ctx, properties);
    }
    return properties;
  }
  function setProperties(ctx, props) {
    const properties = getProperties(ctx);
    Object.entries(props).forEach(([prop, value]) => {
      properties[prop] = value;
    });
    propertyMap.set(ctx, properties);
    return properties;
  }
}
var coalescingContextPropertiesMap = createPropertiesWeakMap((ctx) => ({
  numCoalescingSubscribers: 0
}));
function createCoalesceManager() {
  return {
    remove: removeWork,
    add: addWork,
    isCoalescing
  };
  function removeWork(scope) {
    const numCoalescingSubscribers = coalescingContextPropertiesMap.getProps(scope).numCoalescingSubscribers - 1;
    coalescingContextPropertiesMap.setProps(scope, {
      numCoalescingSubscribers: numCoalescingSubscribers >= 0 ? numCoalescingSubscribers : 0
    });
  }
  function addWork(scope) {
    const numCoalescingSubscribers = coalescingContextPropertiesMap.getProps(scope).numCoalescingSubscribers + 1;
    coalescingContextPropertiesMap.setProps(scope, {
      numCoalescingSubscribers
    });
  }
  function isCoalescing(scope) {
    return coalescingContextPropertiesMap.getProps(scope).numCoalescingSubscribers > 0;
  }
}
function coalesceWith(durationSelector, scope) {
  const _scope = scope || {};
  return (source) => {
    return new Observable((observer) => {
      const rootSubscription = new Subscription();
      rootSubscription.add(source.subscribe(createInnerObserver(observer, rootSubscription)));
      return rootSubscription;
    });
    function createInnerObserver(outerObserver, rootSubscription) {
      let actionSubscription;
      let latestValue;
      const tryEmitLatestValue = () => {
        if (actionSubscription) {
          coalescingManager.remove(_scope);
          if (!coalescingManager.isCoalescing(_scope)) {
            outerObserver.next(latestValue);
          }
        }
      };
      return {
        complete: () => {
          tryEmitLatestValue();
          outerObserver.complete();
        },
        error: (error) => outerObserver.error(error),
        next: (value) => {
          latestValue = value;
          if (!actionSubscription) {
            coalescingManager.add(_scope);
            actionSubscription = durationSelector.subscribe({
              error: (error) => outerObserver.error(error),
              next: () => {
                tryEmitLatestValue();
                actionSubscription?.unsubscribe();
                actionSubscription = void 0;
              },
              complete: () => {
                tryEmitLatestValue();
                actionSubscription = void 0;
              }
            });
            rootSubscription.add(new Subscription(() => {
              tryEmitLatestValue();
              actionSubscription?.unsubscribe();
              actionSubscription = void 0;
            }));
          }
        }
      };
    }
  };
}

// node_modules/@rx-angular/cdk/fesm2022/cdk-internals-scheduler.mjs
function push(heap, node) {
  const index = heap.length;
  heap.push(node);
  siftUp(heap, node, index);
}
function peek(heap) {
  const first = heap[0];
  return first === void 0 ? null : first;
}
function pop(heap) {
  const first = heap[0];
  if (first !== void 0) {
    const last = heap.pop();
    if (last !== first) {
      heap[0] = last;
      siftDown(heap, last, 0);
    }
    return first;
  } else {
    return null;
  }
}
function siftUp(heap, node, i) {
  let index = i;
  while (true) {
    const parentIndex = index - 1 >>> 1;
    const parent = heap[parentIndex];
    if (parent !== void 0 && compare(parent, node) > 0) {
      heap[parentIndex] = node;
      heap[index] = parent;
      index = parentIndex;
    } else {
      return;
    }
  }
}
function siftDown(heap, node, i) {
  let index = i;
  const length = heap.length;
  while (index < length) {
    const leftIndex = (index + 1) * 2 - 1;
    const left = heap[leftIndex];
    const rightIndex = leftIndex + 1;
    const right = heap[rightIndex];
    if (left !== void 0 && compare(left, node) < 0) {
      if (right !== void 0 && compare(right, left) < 0) {
        heap[index] = right;
        heap[rightIndex] = node;
        index = rightIndex;
      } else {
        heap[index] = left;
        heap[leftIndex] = node;
        index = leftIndex;
      }
    } else if (right !== void 0 && compare(right, node) < 0) {
      heap[index] = right;
      heap[rightIndex] = node;
      index = rightIndex;
    } else {
      return;
    }
  }
}
function compare(a, b) {
  const diff = a.sortIndex - b.sortIndex;
  return diff !== 0 ? diff : a.id - b.id;
}
var getCurrentTime;
var hasPerformanceNow = typeof _global.performance === "object" && typeof _global.performance.now === "function";
if (hasPerformanceNow) {
  const localPerformance = _global.performance;
  getCurrentTime = () => localPerformance.now();
} else {
  const localDate = Date;
  const initialTime = localDate.now();
  getCurrentTime = () => localDate.now() - initialTime;
}
var maxSigned31BitInt = 1073741823;
var IMMEDIATE_PRIORITY_TIMEOUT = -1;
var USER_BLOCKING_PRIORITY_TIMEOUT = 250;
var NORMAL_PRIORITY_TIMEOUT = 5e3;
var LOW_PRIORITY_TIMEOUT = 1e4;
var IDLE_PRIORITY_TIMEOUT = maxSigned31BitInt;
var taskQueue = [];
var timerQueue = [];
var taskIdCounter = 1;
var currentTask = null;
var currentPriorityLevel = 3;
var isPerformingWork = false;
var isHostCallbackScheduled = false;
var isHostTimeoutScheduled = false;
var setTimeout = _global.setTimeout;
var clearTimeout = _global.clearTimeout;
var setImmediate = _global.setImmediate;
var messageChannel = _global.MessageChannel;
var defaultZone = {
  run: (fn) => fn()
};
function advanceTimers(currentTime) {
  let timer = peek(timerQueue);
  while (timer !== null) {
    if (timer.callback === null) {
      pop(timerQueue);
    } else if (timer.startTime <= currentTime) {
      pop(timerQueue);
      timer.sortIndex = timer.expirationTime;
      push(taskQueue, timer);
    } else {
      return;
    }
    timer = peek(timerQueue);
  }
}
function handleTimeout(currentTime) {
  isHostTimeoutScheduled = false;
  advanceTimers(currentTime);
  if (!isHostCallbackScheduled) {
    if (peek(taskQueue) !== null) {
      isHostCallbackScheduled = true;
      requestHostCallback(flushWork);
    } else {
      const firstTimer = peek(timerQueue);
      if (firstTimer !== null) {
        requestHostTimeout(handleTimeout, firstTimer.startTime - currentTime);
      }
    }
  }
}
function flushWork(hasTimeRemaining, initialTime) {
  isHostCallbackScheduled = false;
  if (isHostTimeoutScheduled) {
    isHostTimeoutScheduled = false;
    cancelHostTimeout();
  }
  isPerformingWork = true;
  const previousPriorityLevel = currentPriorityLevel;
  try {
    return workLoop(hasTimeRemaining, initialTime);
  } finally {
    currentTask = null;
    currentPriorityLevel = previousPriorityLevel;
    isPerformingWork = false;
  }
}
function workLoop(hasTimeRemaining, initialTime, _currentTask) {
  let currentTime = initialTime;
  if (_currentTask) {
    currentTask = _currentTask;
  } else {
    advanceTimers(currentTime);
    currentTask = peek(taskQueue);
  }
  let zoneChanged = false;
  const hitDeadline = () => currentTask && currentTask.expirationTime > currentTime && (!hasTimeRemaining || shouldYieldToHost());
  if (!hitDeadline()) {
    const ngZone = currentTask.ngZone || defaultZone;
    ngZone.run(() => {
      while (currentTask !== null && !zoneChanged) {
        if (hitDeadline()) {
          break;
        }
        const callback = currentTask.callback;
        if (typeof callback === "function") {
          currentTask.callback = null;
          currentPriorityLevel = currentTask.priorityLevel;
          const didUserCallbackTimeout = currentTask.expirationTime <= currentTime;
          const continuationCallback = callback(didUserCallbackTimeout);
          currentTime = getCurrentTime();
          if (typeof continuationCallback === "function") {
            currentTask.callback = continuationCallback;
          } else {
            if (currentTask === peek(taskQueue)) {
              pop(taskQueue);
            }
          }
          advanceTimers(currentTime);
        } else {
          pop(taskQueue);
        }
        currentTask = peek(taskQueue);
        zoneChanged = currentTask?.ngZone != null && currentTask.ngZone !== ngZone;
      }
    });
  }
  currentTask = currentTask ?? peek(taskQueue);
  currentTime = getCurrentTime();
  if (zoneChanged || currentTask && !hitDeadline()) {
    return workLoop(hasTimeRemaining, currentTime, currentTask);
  }
  if (currentTask !== null) {
    return true;
  } else {
    const firstTimer = peek(timerQueue);
    if (firstTimer !== null) {
      requestHostTimeout(handleTimeout, firstTimer.startTime - currentTime);
    }
    return false;
  }
}
function scheduleCallback(priorityLevel, callback, options) {
  const currentTime = getCurrentTime();
  let startTime;
  if (typeof options === "object" && options !== null) {
    const delay = options.delay;
    if (typeof delay === "number" && delay > 0) {
      startTime = currentTime + delay;
    } else {
      startTime = currentTime;
    }
  } else {
    startTime = currentTime;
  }
  let timeout;
  switch (priorityLevel) {
    case 1:
      timeout = IMMEDIATE_PRIORITY_TIMEOUT;
      break;
    case 2:
      timeout = USER_BLOCKING_PRIORITY_TIMEOUT;
      break;
    case 5:
      timeout = IDLE_PRIORITY_TIMEOUT;
      break;
    case 4:
      timeout = LOW_PRIORITY_TIMEOUT;
      break;
    case 3:
    default:
      timeout = NORMAL_PRIORITY_TIMEOUT;
      break;
  }
  const expirationTime = startTime + timeout;
  const newTask = {
    id: taskIdCounter++,
    callback,
    priorityLevel,
    startTime,
    expirationTime,
    sortIndex: -1,
    ngZone: options?.ngZone || null
  };
  if (startTime > currentTime) {
    newTask.sortIndex = startTime;
    push(timerQueue, newTask);
    if (peek(taskQueue) === null && newTask === peek(timerQueue)) {
      if (isHostTimeoutScheduled) {
        cancelHostTimeout();
      } else {
        isHostTimeoutScheduled = true;
      }
      requestHostTimeout(handleTimeout, startTime - currentTime);
    }
  } else {
    newTask.sortIndex = expirationTime;
    push(taskQueue, newTask);
    if (!isHostCallbackScheduled && !isPerformingWork) {
      isHostCallbackScheduled = true;
      requestHostCallback(flushWork);
    }
  }
  return newTask;
}
function cancelCallback(task) {
  task.callback = null;
}
var isMessageLoopRunning = false;
var scheduledHostCallback = null;
var taskTimeoutID = -1;
var yieldInterval = 16;
var needsPaint = false;
var queueStartTime = -1;
function shouldYieldToHost() {
  if (needsPaint) {
    return true;
  }
  const timeElapsed = getCurrentTime() - queueStartTime;
  if (timeElapsed < yieldInterval) {
    return false;
  }
  return true;
}
function forceFrameRate(fps) {
  if (fps < 0 || fps > 125) {
    if (typeof ngDevMode === "undefined" || ngDevMode) {
      console.error("forceFrameRate takes a positive int between 0 and 125, forcing frame rates higher than 125 fps is not supported");
    }
    return;
  }
  if (fps > 0) {
    yieldInterval = Math.floor(1e3 / fps);
  } else {
    yieldInterval = 5;
  }
  yieldInterval = Math.max(5, yieldInterval - 6);
}
var performWorkUntilDeadline = () => {
  if (scheduledHostCallback !== null) {
    const currentTime = getCurrentTime();
    queueStartTime = currentTime;
    const hasTimeRemaining = true;
    let hasMoreWork = true;
    try {
      hasMoreWork = scheduledHostCallback(hasTimeRemaining, currentTime);
    } finally {
      if (hasMoreWork) {
        schedulePerformWorkUntilDeadline();
      } else {
        isMessageLoopRunning = false;
        scheduledHostCallback = null;
      }
    }
  } else {
    isMessageLoopRunning = false;
  }
  needsPaint = false;
};
var schedulePerformWorkUntilDeadline;
if (typeof setImmediate === "function") {
  schedulePerformWorkUntilDeadline = () => {
    setImmediate(performWorkUntilDeadline);
  };
} else if (typeof messageChannel !== "undefined") {
  const channel = new messageChannel();
  const port = channel.port2;
  channel.port1.onmessage = performWorkUntilDeadline;
  schedulePerformWorkUntilDeadline = () => {
    port.postMessage(null);
  };
} else {
  schedulePerformWorkUntilDeadline = () => {
    setTimeout(performWorkUntilDeadline, 0);
  };
}
function requestHostCallback(callback) {
  scheduledHostCallback = callback;
  if (!isMessageLoopRunning) {
    isMessageLoopRunning = true;
    schedulePerformWorkUntilDeadline();
  }
}
function requestHostTimeout(callback, ms) {
  taskTimeoutID = setTimeout(() => {
    callback(getCurrentTime());
  }, ms);
}
function cancelHostTimeout() {
  clearTimeout(taskTimeoutID);
  taskTimeoutID = -1;
}

// node_modules/@angular/core/fesm2022/rxjs-interop.mjs
function toObservable(source, options) {
  if (ngDevMode && !options?.injector) {
    assertInInjectionContext(toObservable);
  }
  const injector = options?.injector ?? inject(Injector);
  const subject = new ReplaySubject(1);
  const watcher = effect(() => {
    let value;
    try {
      value = source();
    } catch (err) {
      untracked(() => subject.error(err));
      return;
    }
    untracked(() => subject.next(value));
  }, {
    injector,
    manualCleanup: true
  });
  injector.get(DestroyRef).onDestroy(() => {
    watcher.destroy();
    subject.complete();
  });
  return subject.asObservable();
}

// node_modules/@rx-angular/cdk/fesm2022/cdk-internals-core.mjs
function getZoneUnPatchedApi(targetOrName, name) {
  if (typeof targetOrName === "string") {
    name = targetOrName;
    targetOrName = _global;
  }
  return targetOrName["__zone_symbol__" + String(name)] || targetOrName[name];
}
var resolvedPromise = getZoneUnPatchedApi("Promise").resolve();
var resolvedPromise$ = from(resolvedPromise);

// node_modules/@rx-angular/cdk/fesm2022/cdk-coercing.mjs
function coerceObservable(o) {
  return isObservable(o) ? o : of(o);
}
function coerceObservableWith() {
  return (o$) => map(coerceObservable)(o$);
}
function coerceDistinctWith(flattenOperator) {
  flattenOperator = flattenOperator || switchAll();
  return (o$) => o$.pipe(coerceObservableWith(), distinctUntilChanged(), flattenOperator, distinctUntilChanged());
}
function coerceAllFactory(subjectFactory, flattenOperator) {
  const observablesSubject = subjectFactory ? subjectFactory() : new Subject();
  flattenOperator = flattenOperator || switchAll();
  const values$ = observablesSubject.pipe(coerceDistinctWith(flattenOperator));
  return {
    next(observable) {
      observablesSubject.next(observable);
    },
    values$
  };
}

// node_modules/@rx-angular/cdk/fesm2022/cdk-render-strategies.mjs
forceFrameRate(60);
var immediateStrategy = {
  name: "immediate",
  work: (cdRef) => cdRef.detectChanges(),
  behavior: ({
    work,
    scope,
    ngZone
  }) => {
    return (o$) => o$.pipe(scheduleOnQueue(work, {
      ngZone,
      priority: 1,
      scope
    }));
  }
};
var userBlockingStrategy = {
  name: "userBlocking",
  work: (cdRef) => cdRef.detectChanges(),
  behavior: ({
    work,
    scope,
    ngZone
  }) => {
    return (o$) => o$.pipe(scheduleOnQueue(work, {
      ngZone,
      priority: 2,
      scope
    }));
  }
};
var normalStrategy = {
  name: "normal",
  work: (cdRef) => cdRef.detectChanges(),
  behavior: ({
    work,
    scope,
    ngZone
  }) => {
    return (o$) => o$.pipe(scheduleOnQueue(work, {
      ngZone,
      priority: 3,
      scope
    }));
  }
};
var lowStrategy = {
  name: "low",
  work: (cdRef) => cdRef.detectChanges(),
  behavior: ({
    work,
    scope,
    ngZone
  }) => {
    return (o$) => o$.pipe(scheduleOnQueue(work, {
      ngZone,
      priority: 4,
      scope
    }));
  }
};
var idleStrategy = {
  name: "idle",
  work: (cdRef) => cdRef.detectChanges(),
  behavior: ({
    work,
    scope,
    ngZone
  }) => {
    return (o$) => o$.pipe(scheduleOnQueue(work, {
      ngZone,
      priority: 5,
      scope
    }));
  }
};
function scheduleOnQueue(work, options) {
  const scope = options.scope || {};
  return (o$) => o$.pipe(filter(() => !coalescingManager.isCoalescing(scope)), switchMap((v) => new Observable((subscriber) => {
    coalescingManager.add(scope);
    const task = scheduleCallback(options.priority, () => {
      work();
      coalescingManager.remove(scope);
      subscriber.next(v);
    }, {
      delay: options.delay,
      ngZone: options.ngZone
    });
    return () => {
      coalescingManager.remove(scope);
      cancelCallback(task);
    };
  }).pipe(mapTo(v))));
}
var RX_CONCURRENT_STRATEGIES = {
  immediate: immediateStrategy,
  userBlocking: userBlockingStrategy,
  normal: normalStrategy,
  low: lowStrategy,
  idle: idleStrategy
};
var animationFrameTick = () => new Observable((subscriber) => {
  const id = getZoneUnPatchedApi("requestAnimationFrame")(() => {
    subscriber.next(0);
    subscriber.complete();
  });
  return () => {
    getZoneUnPatchedApi("cancelAnimationFrame")(id);
  };
});
var localCredentials = {
  name: "local",
  work: (cdRef, _, notification) => {
    cdRef.detectChanges();
  },
  behavior: ({
    work,
    scope,
    ngZone
  }) => (o$) => o$.pipe(coalesceWith(animationFrameTick(), scope), tap(() => ngZone ? ngZone.run(() => work()) : work()))
};
var noopCredentials = {
  name: "noop",
  work: () => void 0,
  behavior: () => (o$) => o$
};
var nativeCredentials = {
  name: "native",
  work: (cdRef) => cdRef.markForCheck(),
  behavior: ({
    work,
    ngZone
  }) => (o$) => o$.pipe(tap(() => ngZone && !NgZone.isInAngularZone() ? ngZone.run(() => work()) : work()))
};
var RX_NATIVE_STRATEGIES = {
  native: nativeCredentials,
  noop: noopCredentials,
  local: localCredentials
};
var RX_RENDER_STRATEGIES_CONFIG = new InjectionToken("rxa-render-strategies-config");
var RX_RENDER_STRATEGIES_DEFAULTS = {
  primaryStrategy: "normal",
  customStrategies: __spreadValues(__spreadValues({}, RX_NATIVE_STRATEGIES), RX_CONCURRENT_STRATEGIES),
  patchZone: true,
  parent: false
};
function mergeDefaultConfig(cfg) {
  const custom = cfg ? cfg : {
    customStrategies: {}
  };
  return __spreadProps(__spreadValues(__spreadValues({}, RX_RENDER_STRATEGIES_DEFAULTS), custom), {
    customStrategies: __spreadValues(__spreadValues({}, RX_RENDER_STRATEGIES_DEFAULTS.customStrategies), custom.customStrategies)
  });
}
function onStrategy(value, strategy, workFactory, options = {}) {
  return new Observable((subscriber) => {
    subscriber.next(value);
  }).pipe(strategy.behavior({
    work: () => workFactory(value, strategy.work, options),
    scope: options.scope || {},
    ngZone: options.ngZone
  }), catchError((error) => throwError(() => [error, value])), map(() => value), take(1));
}
function strategyHandling(defaultStrategyName, strategies) {
  const hotFlattened = coerceAllFactory(() => new ReplaySubject(1), switchAll());
  return {
    strategy$: hotFlattened.values$.pipe(startWith(defaultStrategyName), nameToStrategyCredentials(strategies, defaultStrategyName), share()),
    next(name) {
      hotFlattened.next(name);
    }
  };
}
function nameToStrategyCredentials(strategies, defaultStrategyName) {
  return (o$) => o$.pipe(map((name) => name && Object.keys(strategies).includes(name) ? strategies[name] : strategies[defaultStrategyName]));
}
var RxStrategyProvider = class _RxStrategyProvider {
  /**
   * @description
   * Returns current `RxAngularConfig` used in the service.
   * Config includes:
   * - strategy that currently in use - `primaryStrategy`
   * - array of custom user defined strategies - `customStrategies`
   * - setting that is responsible for running in our outside of the zone.js - `patchZone`
   */
  get config() {
    return this._cfg;
  }
  /**
   * @description
   * Returns object that contains key-value pairs of strategy names and their credentials (settings) that are available in the service.
   */
  get strategies() {
    return this._strategies$.getValue();
  }
  /**
   * @description
   * Returns an array of strategy names available in the service.
   */
  get strategyNames() {
    return Object.values(this.strategies).map((s) => s.name);
  }
  /**
   * @description
   * Returns current strategy of the service.
   */
  get primaryStrategy() {
    return this._primaryStrategy$.getValue().name;
  }
  /**
   * @description
   * Set's the strategy that will be used by the service.
   */
  set primaryStrategy(strategyName) {
    this._primaryStrategy$.next(this.strategies[strategyName]);
  }
  /**
   * @internal
   */
  constructor(cfg) {
    this._strategies$ = new BehaviorSubject(void 0);
    this._primaryStrategy$ = new BehaviorSubject(void 0);
    this.primaryStrategy$ = this._primaryStrategy$.asObservable();
    this.strategies$ = this._strategies$.asObservable();
    this.strategyNames$ = this.strategies$.pipe(map((strategies) => Object.values(strategies).map((s) => s.name)), shareReplay({
      bufferSize: 1,
      refCount: true
    }));
    this._cfg = mergeDefaultConfig(cfg);
    this._strategies$.next(this._cfg.customStrategies);
    this.primaryStrategy = this.config.primaryStrategy;
  }
  /**
   * @description
   * Allows to schedule a work inside rxjs `pipe`. Accepts the work and configuration options object.
   * - work is any function that should be executed
   * - (optional) options includes strategy, patchZone and scope
   *
   * Scope is by default a subscription but you can also pass `this` and then the scope will be current component.
   * Scope setup is useful if your work is some of the methods of `ChangeDetectorRef`. Only one change detection will be triggered if you have multiple schedules of change detection methods and scope is set to `this`.
   *
   * @example
   * myObservable$.pipe(
   *    this.strategyProvider.scheduleWith(() => myWork(), {strategy: 'idle', patchZone: false})
   * ).subscribe();
   *
   * @return MonoTypeOperatorFunction<R>
   */
  scheduleWith(work, options) {
    const strategy = this.strategies[options?.strategy || this.primaryStrategy];
    const scope = options?.scope || {};
    const _work = getWork(work, options?.patchZone);
    const ngZone = options?.patchZone || void 0;
    return (o$) => o$.pipe(switchMap((v) => onStrategy(v, strategy, (_v) => {
      _work(_v);
    }, {
      scope,
      ngZone
    })));
  }
  /**
   * @description
   * Allows to schedule a work as an observable. Accepts the work and configuration options object.
   * - work is any function that should be executed
   * - (optional) options includes strategy, patchZone and scope
   *
   * Scope is by default a subscription but you can also pass `this` and then the scope will be current component.
   * Scope setup is especially useful if you provide work that will trigger a change detection.
   *
   * @example
   * this.strategyProvider.schedule(() => myWork(), {strategy: 'idle', patchZone: false}).subscribe();
   *
   * @return Observable<R>
   */
  schedule(work, options) {
    const strategy = this.strategies[options?.strategy || this.primaryStrategy];
    const scope = options?.scope || {};
    const _work = getWork(work, options?.patchZone);
    const ngZone = options?.patchZone || void 0;
    let returnVal;
    return onStrategy(null, strategy, () => {
      returnVal = _work();
    }, {
      scope,
      ngZone
    }).pipe(map(() => returnVal));
  }
  /**
   * @description
   * Allows to schedule a change detection cycle. Accepts the ChangeDetectorRef and configuration options object.
   * Options include:
   * - afterCD which is the work that should be executed after change detection cycle.
   * - abortCtrl is an AbortController that you can use to cancel the scheduled cycle.
   *
   * @example
   * this.strategyProvider.scheduleCd(this.changeDetectorRef, {afterCD: myWork()});
   *
   * @return AbortController
   */
  scheduleCD(cdRef, options) {
    const strategy = this.strategies[options?.strategy || this.primaryStrategy];
    const scope = options?.scope || cdRef;
    const abC = options?.abortCtrl || new AbortController();
    const ngZone = options?.patchZone || void 0;
    const work = getWork(() => {
      strategy.work(cdRef, scope);
      if (options?.afterCD) {
        options.afterCD();
      }
    }, options.patchZone);
    onStrategy(null, strategy, () => {
      work();
    }, {
      scope,
      ngZone
    }).pipe(takeUntil(fromEvent(abC.signal, "abort"))).subscribe();
    return abC;
  }
  static {
    this.ɵfac = function RxStrategyProvider_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxStrategyProvider)(ɵɵinject(RX_RENDER_STRATEGIES_CONFIG, 8));
    };
  }
  static {
    this.ɵprov = ɵɵdefineInjectable({
      token: _RxStrategyProvider,
      factory: _RxStrategyProvider.ɵfac,
      providedIn: "root"
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxStrategyProvider, [{
    type: Injectable,
    args: [{
      providedIn: "root"
    }]
  }], () => [{
    type: void 0,
    decorators: [{
      type: Optional
    }, {
      type: Inject,
      args: [RX_RENDER_STRATEGIES_CONFIG]
    }]
  }], null);
})();
function getWork(work, patchZone) {
  let _work = work;
  if (patchZone) {
    _work = (args) => patchZone.run(() => work(args));
  }
  return _work;
}

// node_modules/@rx-angular/cdk/fesm2022/cdk-template.mjs
var computeFirst = ({ count, index }) => index === 0;
var computeLast = ({ count, index }) => index === count - 1;
var computeEven = ({ count, index }) => index % 2 === 0;
var RxDefaultListViewContext = class {
  set $implicit($implicit) {
    this._$implicit = $implicit;
    this._item.next($implicit);
  }
  get $implicit() {
    return this._$implicit;
  }
  get $complete() {
    return this._$complete;
  }
  get $error() {
    return this._$error;
  }
  get $suspense() {
    return this._$suspense;
  }
  get index() {
    return this._context$.getValue().index;
  }
  get count() {
    return this._context$.getValue().count;
  }
  get first() {
    return computeFirst(this._context$.getValue());
  }
  get last() {
    return computeLast(this._context$.getValue());
  }
  get even() {
    return computeEven(this._context$.getValue());
  }
  get odd() {
    return !this.even;
  }
  get index$() {
    return this._context$.pipe(map((c) => c.index), distinctUntilChanged());
  }
  get count$() {
    return this._context$.pipe(map((s) => s.count), distinctUntilChanged());
  }
  get first$() {
    return this._context$.pipe(map(computeFirst), distinctUntilChanged());
  }
  get last$() {
    return this._context$.pipe(map(computeLast), distinctUntilChanged());
  }
  get even$() {
    return this._context$.pipe(map(computeEven), distinctUntilChanged());
  }
  get odd$() {
    return this.even$.pipe(map((even) => !even));
  }
  constructor(item, customProps) {
    this._item = new ReplaySubject(1);
    this.item$ = this._item.asObservable();
    this._context$ = new BehaviorSubject({
      index: -1,
      count: -1
    });
    this.select = (props) => {
      return this.item$.pipe(map((r) => props.reduce((acc, key) => acc?.[key], r)));
    };
    this.$implicit = item;
    if (customProps) {
      this.updateContext(customProps);
    }
  }
  updateContext(newProps) {
    this._context$.next(__spreadValues(__spreadValues({}, this._context$.getValue()), newProps));
  }
};
var RxBaseTemplateNames;
(function(RxBaseTemplateNames2) {
  RxBaseTemplateNames2["error"] = "errorTpl";
  RxBaseTemplateNames2["complete"] = "completeTpl";
  RxBaseTemplateNames2["suspense"] = "suspenseTpl";
})(RxBaseTemplateNames || (RxBaseTemplateNames = {}));

// node_modules/@rx-angular/cdk/fesm2022/cdk-zone-less-browser.mjs
var Promise$1 = getZoneUnPatchedApi("Promise");
function requestAnimationFrame(cb) {
  return getZoneUnPatchedApi("requestAnimationFrame")(cb);
}
function cancelAnimationFrame(id) {
  getZoneUnPatchedApi("cancelAnimationFrame")(id);
}

// node_modules/@rx-angular/template/fesm2022/template-virtual-scrolling.mjs
var _c0 = ["sentinel"];
var _c1 = ["runway"];
var _c2 = ["*"];
function RxVirtualScrollViewportComponent_Conditional_2_Template(rf, ctx) {
  if (rf & 1) {
    ɵɵdomElement(0, "div", 3, 1);
  }
}
var RxVirtualScrollStrategy = class _RxVirtualScrollStrategy {
  constructor() {
    this.viewRenderCallback = new Subject();
  }
  /** @internal */
  get isStable() {
    return of(true);
  }
  /** @internal */
  getElement(view) {
    if (this.nodeIndex !== void 0) {
      return view.rootNodes[this.nodeIndex];
    }
    const rootNode = view.rootNodes[0];
    this.nodeIndex = rootNode instanceof HTMLElement ? 0 : 1;
    return view.rootNodes[this.nodeIndex];
  }
  static {
    this.ɵfac = function RxVirtualScrollStrategy_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxVirtualScrollStrategy)();
    };
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _RxVirtualScrollStrategy
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxVirtualScrollStrategy, [{
    type: Directive
  }], null, null);
})();
var RxVirtualScrollViewport = class _RxVirtualScrollViewport {
  static {
    this.ɵfac = function RxVirtualScrollViewport_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxVirtualScrollViewport)();
    };
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _RxVirtualScrollViewport
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxVirtualScrollViewport, [{
    type: Directive
  }], null, null);
})();
var RxVirtualViewRepeater = class _RxVirtualViewRepeater {
  static {
    this.ɵfac = function RxVirtualViewRepeater_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxVirtualViewRepeater)();
    };
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _RxVirtualViewRepeater
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxVirtualViewRepeater, [{
    type: Directive
  }], null, null);
})();
var RxVirtualForViewContext = class extends RxDefaultListViewContext {
  constructor(item, rxVirtualForOf, customProps) {
    super(item, customProps);
    this.rxVirtualForOf = rxVirtualForOf;
  }
};
var RxVirtualScrollElement = class _RxVirtualScrollElement {
  static {
    this.ɵfac = function RxVirtualScrollElement_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxVirtualScrollElement)();
    };
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _RxVirtualScrollElement
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxVirtualScrollElement, [{
    type: Directive
  }], null, null);
})();
function toBoolean(input) {
  return input != null && `${input}` !== "false";
}
function unpatchedAnimationFrameTick() {
  return new Observable((observer) => {
    const tick = requestAnimationFrame(() => {
      observer.next();
      observer.complete();
    });
    return () => {
      cancelAnimationFrame(tick);
    };
  });
}
function unpatchedMicroTask() {
  return from(Promise$1.resolve());
}
function unpatchedScroll(el) {
  return new Observable((observer) => {
    const listener = () => observer.next();
    getZoneUnPatchedApi(el, "addEventListener").call(el, "scroll", listener, {
      passive: true
    });
    return () => {
      getZoneUnPatchedApi(el, "removeEventListener").call(el, "scroll", listener, {
        passive: true
      });
    };
  });
}
function parseScrollTopBoundaries(scrollTop, offset, contentSize, containerSize) {
  const scrollTopWithOutOffset = scrollTop - offset;
  const maxSize = Math.max(contentSize - containerSize, containerSize);
  const maxScrollTop = Math.max(contentSize, containerSize);
  const adjustedScrollTop = Math.max(0, scrollTopWithOutOffset);
  const scrollTopAfterOffset = adjustedScrollTop - maxSize;
  return {
    scrollTopWithOutOffset,
    scrollTopAfterOffset,
    scrollTop: Math.min(adjustedScrollTop, maxScrollTop)
  };
}
function calculateVisibleContainerSize(containerSize, scrollTopWithOutOffset, scrollTopAfterOffset) {
  let clamped = containerSize;
  if (scrollTopWithOutOffset < 0) {
    clamped = Math.max(0, containerSize + scrollTopWithOutOffset);
  } else if (scrollTopAfterOffset > 0) {
    clamped = Math.max(0, containerSize - scrollTopAfterOffset);
  }
  return clamped;
}
var RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS = new InjectionToken("rx-virtual-scrolling-default-options", {
  providedIn: "root",
  factory: RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS_FACTORY
});
function RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS_FACTORY() {
  return {
    runwayItems: DEFAULT_RUNWAY_ITEMS,
    runwayItemsOpposite: DEFAULT_RUNWAY_ITEMS_OPPOSITE,
    templateCacheSize: DEFAULT_TEMPLATE_CACHE_SIZE,
    itemSize: DEFAULT_ITEM_SIZE
  };
}
var DEFAULT_TEMPLATE_CACHE_SIZE = 20;
var DEFAULT_ITEM_SIZE = 50;
var DEFAULT_RUNWAY_ITEMS = 10;
var DEFAULT_RUNWAY_ITEMS_OPPOSITE = 2;
var RxaResizeObserver = class _RxaResizeObserver {
  constructor() {
    this.resizeObserver = new ResizeObserver((events) => {
      this.viewsResized$.next(events);
    });
    this.viewsResized$ = new Subject();
  }
  observeElement(element, options) {
    this.resizeObserver.observe(element, options);
    return new Observable((observer) => {
      const inner = this.viewsResized$.subscribe((events) => {
        const event = events.find((event2) => event2.target === element);
        if (event) {
          observer.next(event);
        }
      });
      return () => {
        this.resizeObserver.unobserve(element);
        inner.unsubscribe();
      };
    });
  }
  destroy() {
    this.resizeObserver.disconnect();
  }
  static {
    this.ɵfac = function RxaResizeObserver_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxaResizeObserver)();
    };
  }
  static {
    this.ɵprov = ɵɵdefineInjectable({
      token: _RxaResizeObserver,
      factory: _RxaResizeObserver.ɵfac
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxaResizeObserver, [{
    type: Injectable
  }], null, null);
})();
var defaultSizeExtract = (entry) => entry.borderBoxSize[0].blockSize;
var AutoSizeVirtualScrollStrategy = class _AutoSizeVirtualScrollStrategy extends RxVirtualScrollStrategy {
  constructor() {
    super(...arguments);
    this.defaults = inject(RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS, {
      optional: true
    });
    this.runwayItems = this.defaults?.runwayItems ?? 10;
    this.runwayItemsOpposite = this.defaults?.runwayItemsOpposite ?? 2;
    this.tombstoneSize = this.defaults?.itemSize ?? 50;
    this.withResizeObserver = true;
    this.appendOnly = false;
    this.withSyncScrollbar = false;
    this.keepScrolledIndexOnPrepend = false;
    this.viewport = null;
    this.viewRepeater = null;
    this._contentSize$ = new ReplaySubject(1);
    this.contentSize$ = this._contentSize$.asObservable();
    this._contentSize = 0;
    this._renderedRange$ = new Subject();
    this.renderedRange$ = this._renderedRange$.asObservable();
    this._renderedRange = {
      start: 0,
      end: 0
    };
    this.positionedRange = {
      start: 0,
      end: 0
    };
    this._scrolledIndex$ = new ReplaySubject(1);
    this.scrolledIndex$ = this._scrolledIndex$.pipe(distinctUntilChanged());
    this.scrollToTrigger$ = new Subject();
    this._scrolledIndex = 0;
    this._scrollToIndex = null;
    this.containerSize = 0;
    this.contentLength = 0;
    this._virtualItems = [];
    this.scrollTop = 0;
    this.scrollTopWithOutOffset = 0;
    this.scrollTopAfterOffset = 0;
    this.viewportOffset = 0;
    this.direction = "down";
    this.anchorScrollTop = 0;
    this.anchorItem = {
      index: 0,
      offset: 0
    };
    this.lastScreenItem = {
      index: 0,
      offset: 0
    };
    this.waitForScroll = false;
    this.isStable$ = new ReplaySubject(1);
    this.detached$ = new Subject();
    this.resizeObserver = inject(RxaResizeObserver, {
      self: true
    });
    this.recalculateRange$ = new Subject();
  }
  /** @internal */
  set contentSize(size) {
    this._contentSize = size;
    this._contentSize$.next(size);
  }
  get contentSize() {
    return this._contentSize;
  }
  /** @internal */
  set renderedRange(range) {
    this._renderedRange = range;
    this._renderedRange$.next(range);
  }
  /** @internal */
  get renderedRange() {
    return this._renderedRange;
  }
  /** @internal */
  get scrolledIndex() {
    return this._scrolledIndex;
  }
  /** @internal */
  set scrolledIndex(index) {
    this._scrolledIndex = index;
    this._scrolledIndex$.next(index);
  }
  /** @internal */
  until$() {
    return (o$) => o$.pipe(takeUntil(this.detached$));
  }
  /** @internal */
  get extractSize() {
    return this.resizeObserverConfig?.extractSize ?? defaultSizeExtract;
  }
  /** @internal */
  get isStable() {
    return this.isStable$.pipe(filter((w) => w));
  }
  /** @internal */
  ngOnChanges(changes) {
    if (changes["runwayItemsOpposite"] && !changes["runwayItemsOpposite"].firstChange || changes["runwayItems"] && !changes["runwayItems"].firstChange) {
      this.recalculateRange$.next();
    }
    if (changes["withSyncScrollbar"]) {
      this.updateScrollElementClass();
    }
  }
  /** @internal */
  ngOnDestroy() {
    this.detach();
  }
  /** @internal */
  attach(viewport, viewRepeater) {
    this.viewport = viewport;
    this.viewRepeater = viewRepeater;
    this.updateScrollElementClass();
    this.maintainVirtualItems();
    this.calcRenderedRange();
    this.positionElements();
    this.listenToScrollTrigger();
  }
  /** @internal */
  detach() {
    this.updateScrollElementClass(false);
    this.viewport = null;
    this.viewRepeater = null;
    this._virtualItems = [];
    this.resizeObserver.destroy();
    this.detached$.next();
  }
  scrollToIndex(index, behavior) {
    const _index = Math.min(Math.max(index, 0), Math.max(0, this.contentLength - 1));
    if (_index !== this.scrolledIndex) {
      const scrollTop = this.calcInitialPosition(_index);
      this._scrollToIndex = _index;
      this.scrollToTrigger$.next({
        scrollTop,
        behavior
      });
    }
  }
  scrollTo(scrollTo, behavior) {
    this.waitForScroll = scrollTo !== this.scrollTop && this.contentSize > this.containerSize;
    if (this.waitForScroll) {
      this.isStable$.next(false);
    }
    this.viewport.scrollTo(this.viewportOffset + scrollTo, behavior);
  }
  /**
   * starts the subscriptions that maintain the virtualItems array on changes
   * to the underlying dataset
   * @internal
   */
  maintainVirtualItems() {
    this.viewport.containerRect$.pipe(map(({
      width
    }) => width), distinctUntilChanged(), filter(() => this.renderedRange.end > 0 && this._virtualItems.length > 0), this.until$()).subscribe(() => {
      let i = 0;
      while (i < this.renderedRange.start) {
        this._virtualItems[i].cached = false;
        i++;
      }
      i = this.renderedRange.end;
      while (i < this.contentLength - 1) {
        this._virtualItems[i].cached = false;
        i++;
      }
    });
    const itemCache = /* @__PURE__ */ new Map();
    const trackBy = this.viewRepeater._trackBy ?? ((i, item) => item);
    this.renderedRange$.pipe(pairwise(), this.until$()).subscribe(([oldRange, newRange]) => {
      let i = oldRange.start;
      if (i < this._virtualItems.length) {
        for (i; i < Math.min(this._virtualItems.length, oldRange.end); i++) {
          if (i < newRange.start || i >= newRange.end) {
            this._virtualItems[i].position = void 0;
          }
        }
      }
    });
    this.viewRepeater.values$.pipe(this.until$(), tap((values) => {
      const dataArr = Array.isArray(values) ? values : values ? Array.from(values) : [];
      const existingIds = /* @__PURE__ */ new Set();
      let size = 0;
      const dataLength = dataArr.length;
      const virtualItems = new Array(dataLength);
      let anchorItemIndex = this.anchorItem.index;
      const keepScrolledIndexOnPrepend = this.keepScrolledIndexOnPrepend && dataArr.length > 0 && itemCache.size > 0;
      for (let i = 0; i < dataLength; i++) {
        const item = dataArr[i];
        const id = trackBy(i, item);
        const cachedItem = itemCache.get(id);
        if (cachedItem === void 0) {
          virtualItems[i] = {
            size: 0
          };
          itemCache.set(id, {
            item: dataArr[i],
            index: i
          });
          if (i <= anchorItemIndex) {
            anchorItemIndex++;
          }
        } else if (cachedItem.index !== i) {
          virtualItems[i] = this._virtualItems[cachedItem.index];
          virtualItems[i].position = void 0;
          itemCache.set(id, {
            item: dataArr[i],
            index: i
          });
        } else {
          virtualItems[i] = this._virtualItems[i];
          if (!this.withResizeObserver || i < this.renderedRange.start || i >= this.renderedRange.end) {
            virtualItems[i].cached = false;
          }
          itemCache.set(id, {
            item: dataArr[i],
            index: i
          });
        }
        existingIds.add(id);
        size += virtualItems[i].size || this.tombstoneSize;
      }
      this._virtualItems = virtualItems;
      if (itemCache.size > dataLength) {
        itemCache.forEach((v, k) => {
          if (!existingIds.has(k)) {
            itemCache.delete(k);
          }
        });
      }
      existingIds.clear();
      this.contentLength = dataLength;
      if (keepScrolledIndexOnPrepend && this.anchorItem.index !== anchorItemIndex) {
        this.scrollToIndex(anchorItemIndex);
      } else if (dataLength === 0) {
        this.anchorItem = {
          index: 0,
          offset: 0
        };
        this._renderedRange = {
          start: 0,
          end: 0
        };
        this.scrollTo(0);
        this.scrollTop = this.anchorScrollTop = 0;
      } else if (dataLength < this._renderedRange.end) {
        this.anchorItem = this.calculateAnchoredItem({
          index: dataLength,
          offset: 0
        }, Math.max(-size, -calculateVisibleContainerSize(this.containerSize, this.scrollTopWithOutOffset, this.scrollTopAfterOffset)));
        this.calcAnchorScrollTop();
        this._renderedRange = {
          start: Math.max(0, this.anchorItem.index - this.runwayItems),
          end: dataLength
        };
        this.scrollTo(size);
        this.scrollTop = this.anchorScrollTop;
      }
      this.contentSize = size;
    }), finalize(() => itemCache.clear())).subscribe();
  }
  /**
   * listen to triggers that should change the renderedRange
   * @internal
   */
  calcRenderedRange() {
    let removeScrollAnchorOnNextScroll = false;
    const onlyTriggerWhenStable = () => (o$) => o$.pipe(filter(() => this.renderedRange.end === 0 || this.scrollTop === this.anchorScrollTop && this._scrollToIndex === null));
    combineLatest([this.viewport.containerRect$.pipe(map(({
      height
    }) => {
      this.containerSize = height;
      return height;
    }), distinctUntilChanged(), onlyTriggerWhenStable()), this.viewport.elementScrolled$.pipe(startWith(void 0), tap(() => {
      this.viewportOffset = this.viewport.measureOffset();
      const {
        scrollTop,
        scrollTopWithOutOffset,
        scrollTopAfterOffset
      } = parseScrollTopBoundaries(this.viewport.getScrollTop(), this.viewportOffset, this._contentSize, this.containerSize);
      this.direction = scrollTopWithOutOffset > this.scrollTopWithOutOffset ? "down" : "up";
      this.scrollTopWithOutOffset = scrollTopWithOutOffset;
      this.scrollTopAfterOffset = scrollTopAfterOffset;
      this.scrollTop = scrollTop;
      if (removeScrollAnchorOnNextScroll) {
        this._scrollToIndex = null;
        removeScrollAnchorOnNextScroll = false;
      } else {
        removeScrollAnchorOnNextScroll = this._scrollToIndex !== null;
      }
      this.waitForScroll = false;
    })), this._contentSize$.pipe(distinctUntilChanged(), onlyTriggerWhenStable()), this.recalculateRange$.pipe(onlyTriggerWhenStable(), startWith(void 0))]).pipe(
      // make sure to not over calculate things by coalescing all triggers to the next microtask
      coalesceWith(unpatchedMicroTask()),
      map(() => {
        const range = {
          start: 0,
          end: 0
        };
        const delta = this.scrollTop - this.anchorScrollTop;
        if (this.scrollTop === 0) {
          this.anchorItem = {
            index: 0,
            offset: 0
          };
        } else {
          this.anchorItem = this.calculateAnchoredItem(this.anchorItem, delta);
        }
        this.anchorScrollTop = this.scrollTop;
        this.scrolledIndex = this.anchorItem.index;
        this.lastScreenItem = this.calculateAnchoredItem(this.anchorItem, calculateVisibleContainerSize(this.containerSize, this.scrollTopWithOutOffset, this.scrollTopAfterOffset));
        if (this.direction === "up") {
          range.start = Math.max(0, this.anchorItem.index - this.runwayItems);
          range.end = Math.min(this.contentLength, this.lastScreenItem.index + this.runwayItemsOpposite);
        } else {
          range.start = Math.max(0, this.anchorItem.index - this.runwayItemsOpposite);
          range.end = Math.min(this.contentLength, this.lastScreenItem.index + this.runwayItems);
        }
        if (this.appendOnly) {
          range.start = Math.min(this._renderedRange.start, range.start);
          range.end = Math.max(this._renderedRange.end, range.end);
        }
        return range;
      })
    ).pipe(this.until$()).subscribe((range) => {
      this.renderedRange = range;
      this.isStable$.next(!this.waitForScroll);
    });
  }
  /**
   * position elements after they are created/updated/moved or their dimensions
   * change from other sources
   * @internal
   */
  positionElements() {
    const viewsToObserve$ = new Subject();
    const positionByIterableChange$ = this.viewRepeater.renderingStart$.pipe(switchMap((batchedUpdates) => {
      const initialIndex = batchedUpdates.size ? batchedUpdates.values().next().value + this.renderedRange.start : this.renderedRange.start;
      let position = 0;
      let scrollToAnchorPosition = null;
      return this.viewRepeater.viewRendered$.pipe(tap(({
        view,
        index: viewIndex,
        item
      }) => {
        const itemIndex = view.context.index;
        const [, sizeDiff] = this.updateElementSize(view, itemIndex);
        const virtualItem = this._virtualItems[itemIndex];
        if (itemIndex === initialIndex) {
          this.calcAnchorScrollTop();
          position = this.calcInitialPosition(itemIndex);
          if (initialIndex > this.renderedRange.start && virtualItem.position !== position) {
            let beforePosition = position;
            let i = initialIndex - 1;
            while (i >= this.renderedRange.start) {
              const view2 = this.getViewRef(i - this.renderedRange.start);
              const virtualItem2 = this._virtualItems[i];
              const element = this.getElement(view2);
              beforePosition -= virtualItem2.size;
              virtualItem2.position = beforePosition;
              this.positionElement(element, beforePosition);
              i--;
            }
          }
        } else if (itemIndex < this.anchorItem.index && sizeDiff) {
          this.anchorScrollTop += sizeDiff;
        }
        const size = virtualItem.size;
        if (virtualItem.position !== position) {
          const element = this.getElement(view);
          this.positionElement(element, position);
          virtualItem.position = position;
        }
        if (this._scrollToIndex === itemIndex) {
          scrollToAnchorPosition = position;
        }
        position += size;
        viewsToObserve$.next(view);
        this.viewRenderCallback.next({
          index: itemIndex,
          view,
          item
        });
        const {
          lastPositionedIndex: lastIndex,
          position: newPosition
        } = this.positionUnchangedViews({
          viewIndex,
          itemIndex,
          batchedUpdates,
          position
        });
        position = newPosition;
        this.positionedRange.start = this.renderedRange.start;
        this.positionedRange.end = lastIndex + 1;
      }), coalesceWith(unpatchedMicroTask()), tap(() => {
        this.adjustContentSize(position);
        if (this._scrollToIndex === null) {
          this.maybeAdjustScrollPosition();
        } else if (scrollToAnchorPosition != null) {
          if (scrollToAnchorPosition !== this.anchorScrollTop) {
            if (scrollToAnchorPosition > this.contentSize - this.containerSize) {
              if (this.renderedRange.end === this.positionedRange.end) {
                this._scrollToIndex = null;
                this.scrollTo(this.contentSize);
              }
            } else {
              this._scrollToIndex = null;
              this.scrollTo(scrollToAnchorPosition);
            }
          } else {
            this._scrollToIndex = null;
            this.maybeAdjustScrollPosition();
          }
        }
      }));
    }));
    const positionByResizeObserver$ = viewsToObserve$.pipe(filter(() => this.withResizeObserver), groupBy((viewRef) => viewRef), mergeMap((o$) => o$.pipe(exhaustMap((viewRef) => this.observeViewSize$(viewRef)), tap(([index, viewIndex]) => {
      this.calcAnchorScrollTop();
      let position = this.calcInitialPosition(index);
      let viewIdx = viewIndex;
      if (this._virtualItems[index].position !== position) {
        while (viewIdx > 0) {
          viewIdx--;
          position -= this._virtualItems[this.getViewRef(viewIdx).context.index].size;
        }
      } else {
        viewIdx++;
        position += this._virtualItems[index].size;
      }
      while (viewIdx < this.viewRepeater.viewContainer.length) {
        const view = this.getViewRef(viewIdx);
        const itemIndex = view.context.index;
        const virtualItem = this._virtualItems[itemIndex];
        const element = this.getElement(view);
        this.updateElementSize(view, itemIndex);
        virtualItem.position = position;
        this.positionElement(element, position);
        position += virtualItem.size;
        viewIdx++;
      }
      this.maybeAdjustScrollPosition();
    }))));
    merge(positionByIterableChange$, positionByResizeObserver$).pipe(this.until$()).subscribe();
  }
  /** listen to API initiated scroll triggers (e.g. initialScrollIndex) */
  listenToScrollTrigger() {
    this.scrollToTrigger$.pipe(switchMap((scrollTo) => (
      // wait until containerRect at least emitted once
      this.containerSize === 0 ? this.viewport.containerRect$.pipe(map(() => scrollTo), take(1)) : of(scrollTo)
    )), this.until$()).subscribe(({
      scrollTop,
      behavior
    }) => {
      this.scrollTo(scrollTop, behavior);
    });
  }
  /** @internal */
  adjustContentSize(position) {
    let newContentSize = position;
    for (let i = this.positionedRange.end; i < this._virtualItems.length; i++) {
      newContentSize += this.getItemSize(i);
    }
    this.contentSize = newContentSize;
  }
  /** @internal */
  observeViewSize$(viewRef) {
    const element = this.getElement(viewRef);
    return this.resizeObserver.observeElement(element, this.resizeObserverConfig?.options).pipe(takeWhile((event) => event.target.isConnected && !!this._virtualItems[viewRef.context.index]), map((event) => {
      const index = viewRef.context.index;
      const size = Math.round(this.extractSize(event));
      const diff = size - this._virtualItems[index].size;
      if (diff !== 0) {
        this._virtualItems[index].size = size;
        this._virtualItems[index].cached = true;
        this.contentSize += diff;
        return [index, this.viewRepeater.viewContainer.indexOf(viewRef)];
      }
      return null;
    }), filter((diff) => diff !== null && diff[0] >= this.positionedRange.start && diff[0] < this.positionedRange.end), takeUntil(merge(this.viewRepeater.viewRendered$, this.viewRepeater.renderingStart$).pipe(tap(() => {
      const index = viewRef.context.index;
      if (this._virtualItems[index] && (index < this.renderedRange.start || index >= this.renderedRange.end)) {
        this._virtualItems[index].position = void 0;
      }
    }), filter(() => this.viewRepeater.viewContainer.indexOf(viewRef) === -1))));
  }
  /**
   * @internal
   * heavily inspired by
   *   https://github.com/GoogleChromeLabs/ui-element-samples/blob/gh-pages/infinite-scroller/scripts/infinite-scroll.js
   */
  calculateAnchoredItem(initialAnchor, delta) {
    if (delta === 0) return initialAnchor;
    delta += initialAnchor.offset;
    let i = initialAnchor.index;
    const items = this._virtualItems;
    if (delta < 0) {
      while (delta < 0 && i > 0) {
        delta += this.getItemSize(i - 1);
        i--;
      }
    } else {
      while (delta > 0 && i < items.length && this.getItemSize(i) <= delta) {
        delta -= this.getItemSize(i);
        i++;
      }
    }
    return {
      index: Math.min(i, items.length),
      offset: delta
    };
  }
  /** @internal */
  positionUnchangedViews({
    viewIndex,
    itemIndex,
    batchedUpdates,
    position
  }) {
    let _viewIndex = viewIndex + 1;
    let index = itemIndex + 1;
    let lastPositionedIndex = itemIndex;
    while (!batchedUpdates.has(_viewIndex) && index < this.renderedRange.end) {
      const virtualItem = this._virtualItems[index];
      if (position !== virtualItem.position) {
        const view = this.getViewRef(_viewIndex);
        const element = this.getElement(view);
        this.positionElement(element, position);
        virtualItem.position = position;
      }
      position += virtualItem.size;
      lastPositionedIndex = index;
      index++;
      _viewIndex++;
    }
    return {
      position,
      lastPositionedIndex
    };
  }
  /**
   * Adjust the scroll position when the anchorScrollTop differs from
   * the actual scrollTop.
   * Trigger a range recalculation if there is empty space
   *
   * @internal
   */
  maybeAdjustScrollPosition() {
    if (this.anchorScrollTop !== this.scrollTop) {
      this.scrollTo(this.anchorScrollTop);
    }
  }
  /** @internal */
  calcAnchorScrollTop() {
    this.anchorScrollTop = 0;
    for (let i = 0; i < this.anchorItem.index; i++) {
      this.anchorScrollTop += this.getItemSize(i);
    }
    this.anchorScrollTop += this.anchorItem.offset;
  }
  /** @internal */
  calcInitialPosition(start) {
    let pos = this.anchorScrollTop - this.anchorItem.offset;
    let i = this.anchorItem.index;
    while (i > start) {
      const itemSize = this.getItemSize(i - 1);
      pos -= itemSize;
      i--;
    }
    while (i < start) {
      const itemSize = this.getItemSize(i);
      pos += itemSize;
      i++;
    }
    return pos;
  }
  /** @internal */
  getViewRef(index) {
    return this.viewRepeater.viewContainer.get(index);
  }
  /** @internal */
  updateElementSize(view, index) {
    const oldSize = this.getItemSize(index);
    const isCached = this._virtualItems[index].cached;
    const size = isCached ? oldSize : this.getElementSize(this.getElement(view)) || this.tombstoneSize;
    this._virtualItems[index].size = size;
    this._virtualItems[index].cached = true;
    return [size, size - oldSize];
  }
  /** @internal */
  getItemSize(index) {
    return this._virtualItems[index].size || this.tombstoneSize;
  }
  /** @internal */
  getElementSize(element) {
    return element.offsetHeight;
  }
  /** @internal */
  positionElement(element, scrollTop) {
    element.style.position = "absolute";
    element.style.transform = `translateY(${scrollTop}px)`;
  }
  /** @internal */
  updateScrollElementClass(force = this.withSyncScrollbar) {
    const scrollElement = this.viewport?.getScrollElement?.();
    if (!!scrollElement && scrollElement.classList.contains("rx-virtual-scroll-element")) {
      scrollElement.classList.toggle("rx-virtual-scroll-element--withSyncScrollbar", force);
    }
  }
  static {
    this.ɵfac = /* @__PURE__ */ (() => {
      let ɵAutoSizeVirtualScrollStrategy_BaseFactory;
      return function AutoSizeVirtualScrollStrategy_Factory(__ngFactoryType__) {
        return (ɵAutoSizeVirtualScrollStrategy_BaseFactory || (ɵAutoSizeVirtualScrollStrategy_BaseFactory = ɵɵgetInheritedFactory(_AutoSizeVirtualScrollStrategy)))(__ngFactoryType__ || _AutoSizeVirtualScrollStrategy);
      };
    })();
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _AutoSizeVirtualScrollStrategy,
      selectors: [["rx-virtual-scroll-viewport", "autosize", ""]],
      inputs: {
        runwayItems: "runwayItems",
        runwayItemsOpposite: "runwayItemsOpposite",
        tombstoneSize: "tombstoneSize",
        resizeObserverConfig: "resizeObserverConfig",
        withResizeObserver: [2, "withResizeObserver", "withResizeObserver", toBoolean],
        appendOnly: [2, "appendOnly", "appendOnly", toBoolean],
        withSyncScrollbar: [2, "withSyncScrollbar", "withSyncScrollbar", toBoolean],
        keepScrolledIndexOnPrepend: [2, "keepScrolledIndexOnPrepend", "keepScrolledIndexOnPrepend", toBoolean]
      },
      features: [ɵɵProvidersFeature([{
        provide: RxVirtualScrollStrategy,
        useExisting: _AutoSizeVirtualScrollStrategy
      }, RxaResizeObserver]), ɵɵInheritDefinitionFeature, ɵɵNgOnChangesFeature]
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(AutoSizeVirtualScrollStrategy, [{
    type: Directive,
    args: [{
      selector: "rx-virtual-scroll-viewport[autosize]",
      providers: [{
        provide: RxVirtualScrollStrategy,
        useExisting: AutoSizeVirtualScrollStrategy
      }, RxaResizeObserver],
      standalone: true
    }]
  }], null, {
    runwayItems: [{
      type: Input
    }],
    runwayItemsOpposite: [{
      type: Input
    }],
    tombstoneSize: [{
      type: Input
    }],
    resizeObserverConfig: [{
      type: Input
    }],
    withResizeObserver: [{
      type: Input,
      args: [{
        transform: toBoolean
      }]
    }],
    appendOnly: [{
      type: Input,
      args: [{
        transform: toBoolean
      }]
    }],
    withSyncScrollbar: [{
      type: Input,
      args: [{
        transform: toBoolean
      }]
    }],
    keepScrolledIndexOnPrepend: [{
      type: Input,
      args: [{
        transform: toBoolean
      }]
    }]
  });
})();
var defaultItemSize = () => DEFAULT_ITEM_SIZE;
var DynamicSizeVirtualScrollStrategy = class _DynamicSizeVirtualScrollStrategy extends RxVirtualScrollStrategy {
  constructor() {
    super(...arguments);
    this.defaults = inject(RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS, {
      optional: true
    });
    this.runwayItems = this.defaults?.runwayItems ?? DEFAULT_RUNWAY_ITEMS;
    this.runwayItemsOpposite = this.defaults?.runwayItemsOpposite ?? DEFAULT_RUNWAY_ITEMS_OPPOSITE;
    this.appendOnly = false;
    this.keepScrolledIndexOnPrepend = false;
    this._itemSizeFn = defaultItemSize;
    this.waitForScroll = false;
    this.isStable$ = new ReplaySubject(1);
    this.viewport = null;
    this.viewRepeater = null;
    this._contentSize$ = new ReplaySubject(1);
    this.contentSize$ = this._contentSize$.asObservable();
    this._contentSize = 0;
    this._renderedRange$ = new ReplaySubject(1);
    this.renderedRange$ = this._renderedRange$.asObservable();
    this._renderedRange = {
      start: 0,
      end: 0
    };
    this._scrolledIndex$ = new ReplaySubject(1);
    this.scrolledIndex$ = this._scrolledIndex$.pipe(distinctUntilChanged());
    this._scrolledIndex = 0;
    this.containerSize = 0;
    this._virtualItems = [];
    this.scrollTop = 0;
    this.scrollTopWithOutOffset = 0;
    this.scrollTopAfterOffset = 0;
    this.viewportOffset = 0;
    this.direction = "down";
    this.anchorScrollTop = 0;
    this.anchorItem = {
      index: 0,
      offset: 0
    };
    this.lastScreenItem = {
      index: 0,
      offset: 0
    };
    this.detached$ = new Subject();
    this.recalculateRange$ = new Subject();
  }
  /**
   * @description
   * Function returning the size of an item
   */
  set itemSize(fn) {
    if (fn) {
      this._itemSizeFn = fn;
    }
  }
  get itemSize() {
    return this._itemSizeFn;
  }
  /** @internal */
  get isStable() {
    return this.isStable$.pipe(filter((w) => w));
  }
  /** @internal */
  set contentSize(size) {
    this._contentSize = size;
    this._contentSize$.next(size);
  }
  get contentSize() {
    return this._contentSize;
  }
  // range of items where size is known and doesn't need to be re-calculated
  /** @internal */
  set renderedRange(range) {
    this._renderedRange = range;
    this._renderedRange$.next(range);
  }
  /** @internal */
  get renderedRange() {
    return this._renderedRange;
  }
  /** @internal */
  set scrolledIndex(index) {
    this._scrolledIndex = index;
    this._scrolledIndex$.next(index);
  }
  get scrolledIndex() {
    return this._scrolledIndex;
  }
  /** @internal */
  get contentLength() {
    return this._virtualItems.length;
  }
  /** @internal */
  until$() {
    return (o$) => o$.pipe(takeUntil(this.detached$));
  }
  /** @internal */
  ngOnChanges(changes) {
    if (changes["runwayItemsOpposite"] && !changes["runwayItemsOpposite"].firstChange || changes["runwayItems"] && !changes["runwayItems"].firstChange) {
      this.recalculateRange$.next();
    }
  }
  /** @internal */
  ngOnDestroy() {
    this.detach();
  }
  /** @internal */
  attach(viewport, viewRepeater) {
    this.viewport = viewport;
    this.viewRepeater = viewRepeater;
    this.maintainVirtualItems();
    this.calcRenderedRange();
    this.positionElements();
  }
  /** @internal */
  detach() {
    this.viewport = null;
    this.viewRepeater = null;
    this._virtualItems = [];
    this.detached$.next();
  }
  scrollToIndex(index, behavior) {
    const _index = Math.min(Math.max(index, 0), this.contentLength - 1);
    let scrollTo = 0;
    for (let i = 0; i < _index; i++) {
      scrollTo += this._virtualItems[i].size;
    }
    this.scrollTo(scrollTo, behavior);
  }
  scrollTo(scrollTo, behavior) {
    this.waitForScroll = scrollTo !== this.scrollTop && this.contentSize > this.containerSize;
    if (this.waitForScroll) {
      this.isStable$.next(false);
    }
    this.viewport.scrollTo(this.viewportOffset + scrollTo, behavior);
  }
  /** @internal */
  maintainVirtualItems() {
    const valueArray$ = this.viewRepeater.values$.pipe(map((values) => Array.isArray(values) ? values : values != null ? Array.from(values) : []), shareReplay({
      bufferSize: 1,
      refCount: true
    }));
    valueArray$.pipe(this.until$()).subscribe((dataArr) => {
      const dataLength = dataArr.length;
      if (!dataLength) {
        this._virtualItems = [];
        this.contentSize = 0;
        this._renderedRange = {
          start: 0,
          end: 0
        };
        this.anchorItem = {
          index: 0,
          offset: 0
        };
        this.recalculateRange$.next();
      } else {
        let shouldRecalculateRange = false;
        let contentSize = 0;
        for (let i = 0; i < dataArr.length; i++) {
          const oldSize = this._virtualItems[i]?.size;
          const newSize = this.itemSize(dataArr[i]);
          contentSize += newSize;
          if (oldSize === void 0 || oldSize !== newSize) {
            this._virtualItems[i] = {
              size: newSize
            };
            if (!shouldRecalculateRange && (!this.contentSize || i >= this.renderedRange.start && i < this.renderedRange.end)) {
              shouldRecalculateRange = true;
            }
          }
        }
        if (dataLength < this._renderedRange.end) {
          this.anchorItem = this.calculateAnchoredItem({
            index: dataLength,
            offset: 0
          }, -calculateVisibleContainerSize(this.containerSize, this.scrollTopWithOutOffset, this.scrollTopAfterOffset));
          this._renderedRange.start = Math.max(0, this.anchorItem.index - this.runwayItems);
          this._renderedRange.end = dataLength;
          this.calcAnchorScrollTop();
          this.scrollTo(contentSize);
          this.scrollTop = this.anchorScrollTop;
        }
        this.contentSize = contentSize;
        if (shouldRecalculateRange) {
          this.recalculateRange$.next();
        }
      }
    });
    let valueCache = {};
    valueArray$.pipe(
      // TODO: this might cause issues when turning on/off at runtime
      filter(() => this.keepScrolledIndexOnPrepend),
      this.until$()
    ).subscribe((valueArray) => {
      const trackBy = this.viewRepeater._trackBy;
      let scrollTo = this.scrolledIndex;
      const dataLength = valueArray.length;
      const oldDataLength = Object.keys(valueCache).length;
      if (oldDataLength > 0) {
        let i = 0;
        for (i; i <= scrollTo && i < dataLength; i++) {
          if (!valueCache[trackBy(i, valueArray[i])]) {
            scrollTo++;
          }
        }
      }
      valueCache = {};
      valueArray.forEach((v, i) => valueCache[trackBy(i, v)] = v);
      if (scrollTo !== this.scrolledIndex) {
        this.scrollToIndex(scrollTo);
      }
    });
  }
  /** @internal */
  calcRenderedRange() {
    combineLatest([this.viewport.containerRect$.pipe(map(({
      height
    }) => {
      this.containerSize = height;
      return height;
    }), distinctUntilChanged()), this.viewport.elementScrolled$.pipe(startWith(void 0), tap(() => {
      this.viewportOffset = this.viewport.measureOffset();
      const {
        scrollTop,
        scrollTopWithOutOffset,
        scrollTopAfterOffset
      } = parseScrollTopBoundaries(this.viewport.getScrollTop(), this.viewportOffset, this._contentSize, this.containerSize);
      this.direction = scrollTopWithOutOffset > this.scrollTopWithOutOffset ? "down" : "up";
      this.scrollTopWithOutOffset = scrollTopWithOutOffset;
      this.scrollTopAfterOffset = scrollTopAfterOffset;
      this.scrollTop = scrollTop;
      this.waitForScroll = false;
    })), this._contentSize$.pipe(distinctUntilChanged()), this.recalculateRange$.pipe(startWith(void 0))]).pipe(
      // make sure to not over calculate things by coalescing all triggers to the next microtask
      coalesceWith(unpatchedMicroTask()),
      map(() => {
        const range = {
          start: 0,
          end: 0
        };
        const length = this.contentLength;
        const delta = this.scrollTop - this.anchorScrollTop;
        if (this.scrollTop == 0) {
          this.anchorItem = {
            index: 0,
            offset: 0
          };
        } else {
          this.anchorItem = this.calculateAnchoredItem(this.anchorItem, delta);
        }
        this.scrolledIndex = this.anchorItem.index;
        this.anchorScrollTop = this.scrollTop;
        this.lastScreenItem = this.calculateAnchoredItem(this.anchorItem, calculateVisibleContainerSize(this.containerSize, this.scrollTopWithOutOffset, this.scrollTopAfterOffset));
        if (this.direction === "up") {
          range.start = Math.max(0, this.anchorItem.index - this.runwayItems);
          range.end = Math.min(length, this.lastScreenItem.index + this.runwayItemsOpposite);
        } else {
          range.start = Math.max(0, this.anchorItem.index - this.runwayItemsOpposite);
          range.end = Math.min(length, this.lastScreenItem.index + this.runwayItems);
        }
        if (this.appendOnly) {
          range.start = Math.min(this._renderedRange.start, range.start);
          range.end = Math.max(this._renderedRange.end, range.end);
        }
        return range;
      })
    ).pipe(this.until$()).subscribe((range) => {
      this.renderedRange = range;
      this.isStable$.next(!this.waitForScroll);
    });
  }
  /** @internal */
  positionElements() {
    this.viewRepeater.renderingStart$.pipe(switchMap((batchedUpdates) => {
      const renderedRange = this.renderedRange;
      const adjustIndexWith = renderedRange.start;
      const initialIndex = batchedUpdates.size ? batchedUpdates.values().next().value + this.renderedRange.start : this.renderedRange.start;
      let position = this.calcInitialPosition(initialIndex);
      return this.viewRepeater.viewRendered$.pipe(tap(({
        view,
        index: viewIndex,
        item
      }) => {
        const index = viewIndex + adjustIndexWith;
        const size = this.getItemSize(index);
        this.positionElement(this.getElement(view), position);
        position += size;
        this.viewRenderCallback.next({
          index,
          view,
          item
        });
      }));
    }), this.until$()).subscribe();
  }
  /**
   * @internal
   * heavily inspired by
   *   https://github.com/GoogleChromeLabs/ui-element-samples/blob/gh-pages/infinite-scroller/scripts/infinite-scroll.js
   */
  calculateAnchoredItem(initialAnchor, delta) {
    if (delta == 0) return initialAnchor;
    delta += initialAnchor.offset;
    let i = initialAnchor.index;
    const items = this._virtualItems;
    if (delta < 0) {
      while (delta < 0 && i > 0) {
        delta += items[i - 1].size;
        i--;
      }
    } else {
      while (delta > 0 && i < items.length && items[i].size <= delta) {
        delta -= items[i].size;
        i++;
      }
    }
    return {
      index: Math.min(i, items.length),
      offset: delta
    };
  }
  /** @internal */
  calcInitialPosition(start) {
    let pos = this.anchorScrollTop - this.anchorItem.offset;
    let i = this.anchorItem.index;
    while (i > start) {
      const itemSize = this.getItemSize(i - 1);
      pos -= itemSize;
      i--;
    }
    while (i < start) {
      const itemSize = this.getItemSize(i);
      pos += itemSize;
      i++;
    }
    return pos;
  }
  /** @internal */
  calcAnchorScrollTop() {
    this.anchorScrollTop = 0;
    for (let i = 0; i < this.anchorItem.index; i++) {
      this.anchorScrollTop += this.getItemSize(i);
    }
    this.anchorScrollTop += this.anchorItem.offset;
  }
  /** @internal */
  getItemSize(index) {
    return this._virtualItems[index].size;
  }
  /** @internal */
  positionElement(element, scrollTop) {
    element.style.position = "absolute";
    element.style.transform = `translateY(${scrollTop}px)`;
  }
  static {
    this.ɵfac = /* @__PURE__ */ (() => {
      let ɵDynamicSizeVirtualScrollStrategy_BaseFactory;
      return function DynamicSizeVirtualScrollStrategy_Factory(__ngFactoryType__) {
        return (ɵDynamicSizeVirtualScrollStrategy_BaseFactory || (ɵDynamicSizeVirtualScrollStrategy_BaseFactory = ɵɵgetInheritedFactory(_DynamicSizeVirtualScrollStrategy)))(__ngFactoryType__ || _DynamicSizeVirtualScrollStrategy);
      };
    })();
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _DynamicSizeVirtualScrollStrategy,
      selectors: [["rx-virtual-scroll-viewport", "dynamic", ""]],
      inputs: {
        runwayItems: "runwayItems",
        runwayItemsOpposite: "runwayItemsOpposite",
        appendOnly: [2, "appendOnly", "appendOnly", toBoolean],
        keepScrolledIndexOnPrepend: [2, "keepScrolledIndexOnPrepend", "keepScrolledIndexOnPrepend", toBoolean],
        itemSize: [0, "dynamic", "itemSize"]
      },
      features: [ɵɵProvidersFeature([{
        provide: RxVirtualScrollStrategy,
        useExisting: _DynamicSizeVirtualScrollStrategy
      }]), ɵɵInheritDefinitionFeature, ɵɵNgOnChangesFeature]
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(DynamicSizeVirtualScrollStrategy, [{
    type: Directive,
    args: [{
      selector: "rx-virtual-scroll-viewport[dynamic]",
      providers: [{
        provide: RxVirtualScrollStrategy,
        useExisting: DynamicSizeVirtualScrollStrategy
      }],
      standalone: true
    }]
  }], null, {
    runwayItems: [{
      type: Input
    }],
    runwayItemsOpposite: [{
      type: Input
    }],
    appendOnly: [{
      type: Input,
      args: [{
        transform: toBoolean
      }]
    }],
    keepScrolledIndexOnPrepend: [{
      type: Input,
      args: [{
        transform: toBoolean
      }]
    }],
    itemSize: [{
      type: Input,
      args: ["dynamic"]
    }]
  });
})();
var FixedSizeVirtualScrollStrategy = class _FixedSizeVirtualScrollStrategy extends RxVirtualScrollStrategy {
  constructor() {
    super(...arguments);
    this.defaults = inject(RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS, {
      optional: true
    });
    this._itemSize = DEFAULT_ITEM_SIZE;
    this.appendOnly = false;
    this.runwayItems = this.defaults?.runwayItems ?? DEFAULT_RUNWAY_ITEMS;
    this.runwayItemsOpposite = this.defaults?.runwayItemsOpposite ?? DEFAULT_RUNWAY_ITEMS_OPPOSITE;
    this.keepScrolledIndexOnPrepend = false;
    this.runwayStateChanged$ = new Subject();
    this.viewport = null;
    this.viewRepeater = null;
    this._scrolledIndex$ = new ReplaySubject(1);
    this.scrolledIndex$ = this._scrolledIndex$.pipe(distinctUntilChanged());
    this._scrolledIndex = 0;
    this._contentSize$ = new ReplaySubject(1);
    this.contentSize$ = this._contentSize$.asObservable();
    this._contentSize = 0;
    this._renderedRange$ = new ReplaySubject(1);
    this.renderedRange$ = this._renderedRange$.asObservable();
    this._renderedRange = {
      start: 0,
      end: 0
    };
    this.scrollTop = 0;
    this.scrollTopWithOutOffset = 0;
    this.scrollTopAfterOffset = 0;
    this.viewportOffset = 0;
    this.containerSize = 0;
    this.direction = "down";
    this.detached$ = new Subject();
  }
  /**
   * @description
   * The size of the items in the virtually scrolled list
   */
  set itemSize(itemSize) {
    if (typeof itemSize === "number") {
      this._itemSize = itemSize;
    }
  }
  get itemSize() {
    return this._itemSize;
  }
  set scrolledIndex(index) {
    this._scrolledIndex = index;
    this._scrolledIndex$.next(index);
  }
  get scrolledIndex() {
    return this._scrolledIndex;
  }
  set contentSize(size) {
    this._contentSize = size;
    this._contentSize$.next(size);
  }
  set renderedRange(range) {
    this._renderedRange = range;
    this._renderedRange$.next(range);
  }
  get renderedRange() {
    return this._renderedRange;
  }
  /** @internal */
  ngOnChanges(changes) {
    if (changes["runwayItemsOpposite"] && !changes["runwayItemsOpposite"].firstChange || changes["runwayItems"] && !changes["runwayItems"].firstChange) {
      this.runwayStateChanged$.next();
    }
  }
  ngOnDestroy() {
    this.detach();
  }
  attach(viewport, viewRepeater) {
    this.viewport = viewport;
    this.viewRepeater = viewRepeater;
    this.calcRenderedRange();
    this.positionElements();
  }
  detach() {
    this.viewport = null;
    this.viewRepeater = null;
    this.detached$.next();
  }
  positionElements() {
    this.viewRepeater.renderingStart$.pipe(switchMap(() => {
      const start = this.renderedRange.start;
      return this.viewRepeater.viewRendered$.pipe(tap(({
        view,
        index,
        item
      }) => {
        this._setViewPosition(view, (index + start) * this.itemSize);
        this.viewRenderCallback.next({
          view,
          item,
          index
        });
      }));
    }), this.untilDetached$()).subscribe();
  }
  calcRenderedRange() {
    const valueArray$ = this.viewRepeater.values$.pipe(map((values) => Array.isArray(values) ? values : values != null ? Array.from(values) : []), shareReplay({
      bufferSize: 1,
      refCount: true
    }));
    let valueCache = {};
    valueArray$.pipe(
      // TODO: this might cause issues when turning on/off
      filter(() => this.keepScrolledIndexOnPrepend),
      this.untilDetached$()
    ).subscribe((valueArray) => {
      const trackBy = this.viewRepeater._trackBy;
      let scrollTo = this.scrolledIndex;
      const dataLength = valueArray.length;
      const oldDataLength = Object.keys(valueCache).length;
      if (oldDataLength > 0) {
        let i = 0;
        for (i; i <= scrollTo && i < dataLength; i++) {
          if (!valueCache[trackBy(i, valueArray[i])]) {
            scrollTo++;
          }
        }
      }
      valueCache = {};
      valueArray.forEach((v, i) => valueCache[trackBy(i, v)] = v);
      if (scrollTo !== this.scrolledIndex) {
        this.scrollToIndex(scrollTo);
      }
    });
    const dataLengthChanged$ = valueArray$.pipe(map((values) => values.length), distinctUntilChanged(), tap((dataLength) => this.contentSize = dataLength * this.itemSize));
    const onScroll$ = this.viewport.elementScrolled$.pipe(coalesceWith(unpatchedAnimationFrameTick()), startWith(void 0), tap(() => {
      this.viewportOffset = this.viewport.measureOffset();
      const {
        scrollTop,
        scrollTopWithOutOffset,
        scrollTopAfterOffset
      } = parseScrollTopBoundaries(this.viewport.getScrollTop(), this.viewportOffset, this._contentSize, this.containerSize);
      this.direction = scrollTopWithOutOffset > this.scrollTopWithOutOffset ? "down" : "up";
      this.scrollTopWithOutOffset = scrollTopWithOutOffset;
      this.scrollTopAfterOffset = scrollTopAfterOffset;
      this.scrollTop = scrollTop;
    }));
    combineLatest([dataLengthChanged$, this.viewport.containerRect$.pipe(map(({
      height
    }) => {
      this.containerSize = height;
      return height;
    }), distinctUntilChanged()), onScroll$, this.runwayStateChanged$.pipe(startWith(void 0))]).pipe(map(([length]) => {
      const containerSize = calculateVisibleContainerSize(this.containerSize, this.scrollTopWithOutOffset, this.scrollTopAfterOffset);
      const range = {
        start: 0,
        end: 0
      };
      if (this.direction === "up") {
        range.start = Math.floor(Math.max(0, this.scrollTop - this.runwayItems * this.itemSize) / this.itemSize);
        range.end = Math.min(length, Math.ceil((this.scrollTop + containerSize + this.runwayItemsOpposite * this.itemSize) / this.itemSize));
      } else {
        range.start = Math.floor(Math.max(0, this.scrollTop - this.runwayItemsOpposite * this.itemSize) / this.itemSize);
        range.end = Math.min(length, Math.ceil((this.scrollTop + containerSize + this.runwayItems * this.itemSize) / this.itemSize));
      }
      if (this.appendOnly) {
        range.start = Math.min(this._renderedRange.start, range.start);
        range.end = Math.max(this._renderedRange.end, range.end);
      }
      this.scrolledIndex = Math.floor(this.scrollTop / this.itemSize);
      return range;
    }), distinctUntilChanged(({
      start: prevStart,
      end: prevEnd
    }, {
      start,
      end
    }) => prevStart === start && prevEnd === end), this.untilDetached$()).subscribe((range) => this.renderedRange = range);
  }
  scrollToIndex(index, behavior) {
    const scrollTop = this.itemSize * index;
    this.viewport.scrollTo(this.viewportOffset + scrollTop, behavior);
  }
  untilDetached$() {
    return (o$) => o$.pipe(takeUntil(this.detached$));
  }
  _setViewPosition(view, scrollTop) {
    const element = this.getElement(view);
    element.style.position = "absolute";
    element.style.transform = `translateY(${scrollTop}px)`;
  }
  static {
    this.ɵfac = /* @__PURE__ */ (() => {
      let ɵFixedSizeVirtualScrollStrategy_BaseFactory;
      return function FixedSizeVirtualScrollStrategy_Factory(__ngFactoryType__) {
        return (ɵFixedSizeVirtualScrollStrategy_BaseFactory || (ɵFixedSizeVirtualScrollStrategy_BaseFactory = ɵɵgetInheritedFactory(_FixedSizeVirtualScrollStrategy)))(__ngFactoryType__ || _FixedSizeVirtualScrollStrategy);
      };
    })();
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _FixedSizeVirtualScrollStrategy,
      selectors: [["rx-virtual-scroll-viewport", "itemSize", ""]],
      inputs: {
        itemSize: "itemSize",
        appendOnly: [2, "appendOnly", "appendOnly", toBoolean],
        runwayItems: "runwayItems",
        runwayItemsOpposite: "runwayItemsOpposite",
        keepScrolledIndexOnPrepend: [2, "keepScrolledIndexOnPrepend", "keepScrolledIndexOnPrepend", toBoolean]
      },
      features: [ɵɵProvidersFeature([{
        provide: RxVirtualScrollStrategy,
        useExisting: _FixedSizeVirtualScrollStrategy
      }]), ɵɵInheritDefinitionFeature, ɵɵNgOnChangesFeature]
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(FixedSizeVirtualScrollStrategy, [{
    type: Directive,
    args: [{
      selector: "rx-virtual-scroll-viewport[itemSize]",
      providers: [{
        provide: RxVirtualScrollStrategy,
        useExisting: FixedSizeVirtualScrollStrategy
      }],
      standalone: true
    }]
  }], null, {
    itemSize: [{
      type: Input
    }],
    appendOnly: [{
      type: Input,
      args: [{
        transform: toBoolean
      }]
    }],
    runwayItems: [{
      type: Input
    }],
    runwayItemsOpposite: [{
      type: Input
    }],
    keepScrolledIndexOnPrepend: [{
      type: Input,
      args: [{
        transform: toBoolean
      }]
    }]
  });
})();
function createVirtualListTemplateManager({
  viewContainerRef,
  templateRef,
  createViewContext,
  updateViewContext,
  templateCacheSize
}) {
  let _viewCache = [];
  let itemCount = 0;
  return {
    getListChanges,
    setItemCount: (count) => itemCount = count,
    detach: () => {
      for (let i = 0; i < _viewCache.length; i++) {
        _viewCache[i].destroy();
      }
      _viewCache = [];
    }
  };
  function _updateView(item, index, count, contextIndex) {
    const view = viewContainerRef.get(index);
    updateViewContext(item, view, {
      count,
      index: contextIndex
    });
    view.detectChanges();
    return view;
  }
  function _insertView(value, count, adjustIndexWith, currentIndex) {
    currentIndex = currentIndex ?? viewContainerRef.length;
    const contextIndex = currentIndex + adjustIndexWith;
    const cachedView = _insertViewFromCache(currentIndex);
    if (cachedView) {
      updateViewContext(value, cachedView, {
        count,
        index: contextIndex
      });
      cachedView.detectChanges();
      return [currentIndex, cachedView];
    }
    const context = createViewContext(value, {
      count,
      index: contextIndex
    });
    const view = viewContainerRef.createEmbeddedView(templateRef, context, currentIndex);
    view.detectChanges();
    return [currentIndex, view];
  }
  function _detachAndCacheView(index) {
    const detachedView = viewContainerRef.detach(index);
    _maybeCacheView(detachedView);
    detachedView.detectChanges();
  }
  function _moveView(value, adjustedPreviousIndex, currentIndex, count, contextIndex) {
    const oldView = viewContainerRef.get(adjustedPreviousIndex);
    const view = viewContainerRef.move(oldView, currentIndex);
    updateViewContext(value, view, {
      count,
      index: contextIndex
    });
    view.detectChanges();
    return view;
  }
  function _maybeCacheView(view) {
    if (_viewCache.length < templateCacheSize) {
      _viewCache.push(view);
      return true;
    } else {
      const index = viewContainerRef.indexOf(view);
      if (index === -1) {
        view.destroy();
      } else {
        viewContainerRef.remove(index);
      }
      return false;
    }
  }
  function _insertViewFromCache(index) {
    const cachedView = _viewCache.pop();
    if (cachedView) {
      return viewContainerRef.insert(cachedView, index);
    }
    return null;
  }
  function getListChanges(changes, items, count, adjustIndexWith) {
    const changedIdxs = /* @__PURE__ */ new Set();
    const listChanges = [];
    let notifyParent = false;
    let appendedAtEnd = 0;
    const otherMovedIds = [];
    changes.forEachOperation(({
      item,
      previousIndex
    }, adjustedPreviousIndex, currentIndex) => {
      if (previousIndex == null) {
        const index = currentIndex === null ? void 0 : currentIndex;
        listChanges.push([index ?? items.length + appendedAtEnd, () => {
          const [insertedIndex, view] = _insertView(item, count, adjustIndexWith, index);
          return {
            view,
            index: insertedIndex,
            item
          };
        }]);
        if (index === void 0) {
          appendedAtEnd++;
        }
        changedIdxs.add(item);
        notifyParent = true;
      } else if (currentIndex == null) {
        listChanges.push([adjustedPreviousIndex, () => {
          _detachAndCacheView(adjustedPreviousIndex ?? void 0);
          return {
            item
          };
        }, true]);
        notifyParent = true;
      } else if (adjustedPreviousIndex !== null) {
        listChanges.push([currentIndex, () => {
          const view = _moveView(item, adjustedPreviousIndex, currentIndex, count, currentIndex + adjustIndexWith);
          return {
            view,
            index: currentIndex,
            item
          };
        }]);
        otherMovedIds.push(adjustedPreviousIndex);
        changedIdxs.add(item);
        notifyParent = true;
      }
    });
    changes.forEachIdentityChange(({
      item,
      currentIndex
    }) => {
      if (currentIndex != null && !changedIdxs.has(item)) {
        listChanges.push([currentIndex, () => {
          const view = _updateView(item, currentIndex, count, currentIndex + adjustIndexWith);
          return {
            view,
            index: currentIndex,
            item
          };
        }]);
        changedIdxs.add(item);
      }
    });
    for (let i = 0; i < otherMovedIds.length; i++) {
      const itemIndex = otherMovedIds[i];
      const item = items[itemIndex];
      if (item && !changedIdxs.has(item)) {
        changedIdxs.add(item);
        listChanges.push([itemIndex, () => maybeUpdateView(itemIndex, count, itemIndex + adjustIndexWith, item)]);
      }
    }
    if (changedIdxs.size < items.length) {
      for (let i = 0; i < items.length; i++) {
        const item = items[i];
        if (!changedIdxs.has(item)) {
          listChanges.push([i, () => maybeUpdateView(i, count, i + adjustIndexWith, item)]);
        }
      }
    }
    return [listChanges, notifyParent];
  }
  function maybeUpdateView(viewIndex, count, itemIndex, item) {
    const view = viewContainerRef.get(viewIndex);
    if (view.context.count !== count || view.context.index !== itemIndex) {
      return {
        view: _updateView(item, viewIndex, count, itemIndex),
        index: viewIndex,
        item
      };
    }
    return {
      index: viewIndex,
      view,
      item
    };
  }
}
var NG_DEV_MODE$1 = typeof ngDevMode === "undefined" || !!ngDevMode;
var RxVirtualFor = class _RxVirtualFor {
  /**
   * @description
   * The iterable input
   *
   * @example
   * <rx-virtual-scroll-viewport>
   *   <app-hero *rxVirtualFor="heroes$; let hero"
   *     [hero]="hero"></app-hero>
   * </rx-virtual-scroll-viewport>
   *
   * @param potentialSignalOrObservable
   */
  set rxVirtualForOf(potentialSignalOrObservable) {
    if (isSignal(potentialSignalOrObservable)) {
      this.staticValue = void 0;
      this.renderStatic = false;
      this.observables$.next(toObservable(potentialSignalOrObservable, {
        injector: this.injector
      }));
    } else if (!isObservable(potentialSignalOrObservable)) {
      this.staticValue = potentialSignalOrObservable;
      this.renderStatic = true;
    } else {
      this.staticValue = void 0;
      this.renderStatic = false;
      this.observables$.next(potentialSignalOrObservable);
    }
  }
  set rxVirtualForTemplate(value) {
    this._template = value;
  }
  /**
   * @description
   * The rendering strategy to be used to render updates to the DOM.
   * Use it to dynamically manage your rendering strategy. You can switch the strategy
   * imperatively (with a string) or by binding an Observable.
   * The default strategy is `'normal'` if not configured otherwise.
   *
   * @example
   * \@Component({
   *   selector: 'app-root',
   *   template: `
   *     <rx-virtual-scroll-viewport>
   *       <app-hero
   *        autosized
   *        *rxVirtualFor="let hero of heroes$; strategy: strategy"
   *        [hero]="hero"></app-hero>
   *     </rx-virtual-scroll-viewport>
   *   `
   * })
   * export class AppComponent {
   *   strategy = 'low';
   * }
   *
   * // OR
   *
   * \@Component({
   *   selector: 'app-root',
   *   template: `
   *     <rx-virtual-scroll-viewport>
   *       <app-hero
   *        autosized
   *        *rxVirtualFor="let hero of heroes$; strategy: strategy$"
   *        [hero]="hero"></app-hero>
   *     </rx-virtual-scroll-viewport>
   *   `
   * })
   * export class AppComponent {
   *   strategy$ = new BehaviorSubject('immediate');
   * }
   *
   * @param strategyName
   * @see {@link strategies}
   */
  set strategy(strategyName) {
    this.strategyHandler.next(strategyName);
  }
  /*@Input('rxVirtualForTombstone') tombstone: TemplateRef<
   RxVirtualForViewContext<T>
   > | null = null;*/
  /**
   * @description
   * A function or key that defines how to track changes for items in the provided
   * iterable data.
   *
   * When items are added, moved, or removed in the iterable,
   * the directive must re-render the appropriate DOM nodes.
   * To minimize operations on the DOM, only nodes that have changed
   * are re-rendered.
   *
   * By default, `rxVirtualFor` assumes that the object instance identifies
   * the node in the iterable (equality check `===`).
   * When a function or key is supplied, `rxVirtualFor` uses the result to identify the item node.
   *
   * @example
   * \@Component({
   *   selector: 'app-root',
   *   template: `
   *    <rx-virtual-scroll-viewport>
   *      <app-list-item
   *        *rxVirtualFor="
   *          let item of items$;
   *          trackBy: 'id';
   *        "
   *        autosized
   *        [item]="item"
   *      >
   *      </app-list-item>
   *    </rx-virtual-scroll-viewport>
   *   `
   * })
   * export class AppComponent {
   *   items$ = itemService.getItems();
   * }
   *
   * // OR
   *
   * \@Component({
   *   selector: 'app-root',
   *   template: `
   *   <rx-virtual-scroll-viewport>
   *      <app-list-item
   *        *rxVirtualFor="
   *          let item of items$;
   *          trackBy: trackItem;
   *        "
   *        autosized
   *        [item]="item"
   *      >
   *      </app-list-item>
   *    </rx-virtual-scroll-viewport>
   *   `
   * })
   * export class AppComponent {
   *   items$ = itemService.getItems();
   *   trackItem = (idx, item) => item.id;
   * }
   *
   * @param trackByFnOrKey
   */
  set trackBy(trackByFnOrKey) {
    if (NG_DEV_MODE$1 && trackByFnOrKey != null && typeof trackByFnOrKey !== "string" && typeof trackByFnOrKey !== "symbol" && typeof trackByFnOrKey !== "function") {
      throw new Error(`trackBy must be typeof function or keyof T, but received ${JSON.stringify(trackByFnOrKey)}.`);
    }
    if (trackByFnOrKey == null) {
      this._trackBy = null;
    } else {
      this._trackBy = typeof trackByFnOrKey !== "function" ? (i, a) => a[trackByFnOrKey] : trackByFnOrKey;
    }
  }
  /**
   * @description
   * A `Subject` which emits whenever `*rxVirtualFor` finished rendering a
   * set of changes to the view.
   * This enables developers to perform actions exactly at the timing when the
   * updates passed are rendered to the DOM.
   * The `renderCallback` is useful in situations where you rely on specific DOM
   * properties like the `height` of a table after all items got rendered.
   * It is also possible to use the renderCallback in order to determine if a
   * view should be visible or not. This way developers can hide a list as
   * long as it has not finished rendering.
   *
   * The result of the `renderCallback` will contain the currently rendered set
   * of items in the iterable.
   *
   * @example
   * \@Component({
   *   selector: 'app-root',
   *   template: `
   *    <rx-virtual-scroll-viewport>
   *      <app-list-item
   *        *rxVirtualFor="
   *          let item of items$;
   *          trackBy: trackItem;
   *          renderCallback: itemsRendered;
   *        "
   *        autosized
   *        [item]="item"
   *      >
   *      </app-list-item>
   *    </rx-virtual-scroll-viewport>
   *   `
   * })
   * export class AppComponent {
   *   items$: Observable<Item[]> = itemService.getItems();
   *   trackItem = (idx, item) => item.id;
   *   // this emits whenever rxVirtualFor finished rendering changes
   *   itemsRendered = new Subject<Item[]>();
   * }
   *
   * @param renderCallback
   */
  set renderCallback(renderCallback) {
    this._renderCallback = renderCallback;
  }
  /** @internal */
  get template() {
    return this._template || this.templateRef;
  }
  /** @internal */
  static ngTemplateContextGuard(dir, ctx) {
    return true;
  }
  constructor(templateRef) {
    this.templateRef = templateRef;
    this.scrollStrategy = inject(RxVirtualScrollStrategy);
    this.iterableDiffers = inject(IterableDiffers);
    this.cdRef = inject(ChangeDetectorRef);
    this.ngZone = inject(NgZone);
    this.injector = inject(Injector);
    this.viewContainer = inject(ViewContainerRef);
    this.strategyProvider = inject(RxStrategyProvider);
    this.errorHandler = inject(ErrorHandler);
    this.defaults = inject(RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS, {
      optional: true
    });
    this.partiallyFinished = false;
    this.renderStatic = false;
    this.strategyHandler = strategyHandling(this.strategyProvider.primaryStrategy, this.strategyProvider.strategies);
    this.templateCacheSize = this.defaults?.templateCacheSize || DEFAULT_TEMPLATE_CACHE_SIZE;
    this.renderParent = false;
    this.patchZone = this.strategyProvider.config.patchZone;
    this.viewsRendered$ = new Subject();
    this.viewRendered$ = new Subject();
    this.renderingStart$ = new Subject();
    this.observables$ = new ReplaySubject(1);
    this.values$ = this.observables$.pipe(coerceObservableWith(), switchAll(), shareReplay({
      bufferSize: 1,
      refCount: true
    }));
    this._destroy$ = new Subject();
    this._trackBy = null;
  }
  /** @internal */
  ngOnInit() {
    this.values$.pipe(takeUntil(this._destroy$)).subscribe((values) => {
      this.values = values;
    });
    this.templateManager = createVirtualListTemplateManager({
      viewContainerRef: this.viewContainer,
      templateRef: this.template,
      createViewContext: this.createViewContext.bind(this),
      updateViewContext: this.updateViewContext.bind(this),
      templateCacheSize: this.templateCacheSize
    });
    Promise$1.resolve().then(() => {
      this.render().pipe(takeUntil(this._destroy$)).subscribe((v) => {
        this._renderCallback?.next(v);
      });
    });
  }
  /** @internal */
  ngDoCheck() {
    if (this.renderStatic) {
      this.observables$.next(this.staticValue);
    }
  }
  /** @internal */
  ngOnDestroy() {
    this._destroy$.next();
    this.templateManager.detach();
  }
  render() {
    return combineLatest([this.values$.pipe(map((values) => Array.isArray(values) ? values : values != null ? Array.from(values) : [])), this.scrollStrategy.renderedRange$.pipe(distinctUntilChanged((oldRange, newRange) => oldRange.start === newRange.start && oldRange.end === newRange.end)), this.strategyHandler.strategy$.pipe(distinctUntilChanged())]).pipe(switchMap(([items, range, strategy]) => (
      // wait for scrollStrategy to be stable until computing new state
      this.scrollStrategy.isStable.pipe(
        take(1),
        // map iterable to latest diff
        switchMap(() => {
          const iterable = items.slice(range.start, range.end);
          const differ = this.getDiffer(iterable);
          let changes = null;
          if (differ) {
            if (this.partiallyFinished) {
              const currentIterable = [];
              for (let i = 0, ilen = this.viewContainer.length; i < ilen; i++) {
                const viewRef = this.viewContainer.get(i);
                currentIterable[i] = viewRef.context.$implicit;
              }
              differ.diff(currentIterable);
            }
            changes = differ.diff(iterable);
          }
          if (!changes) {
            return NEVER;
          }
          const listChanges = this.templateManager.getListChanges(changes, iterable, items.length, range.start);
          const updates = listChanges[0].sort((a, b) => a[0] - b[0]);
          const indicesToPosition = /* @__PURE__ */ new Set();
          const insertedOrRemoved = listChanges[1];
          const work$ = updates.map(([index, work, removed]) => {
            if (!removed) {
              indicesToPosition.add(index);
            }
            return onStrategy(null, strategy, () => {
              const update = work();
              if (update.view) {
                this.viewRendered$.next(update);
              }
            }, {
              ngZone: this.patchZone ? this.ngZone : void 0
            });
          });
          this.partiallyFinished = true;
          const notifyParent = insertedOrRemoved && this.renderParent;
          this.renderingStart$.next(indicesToPosition);
          return combineLatest(
            // emit after all changes are rendered
            work$.length > 0 ? work$ : [of(iterable)]
          ).pipe(tap(() => {
            this.templateManager.setItemCount(items.length);
            this.partiallyFinished = false;
            const viewsRendered = [];
            const end = this.viewContainer.length;
            let i = 0;
            for (i; i < end; i++) {
              viewsRendered.push(this.viewContainer.get(i));
            }
            this.viewsRendered$.next(viewsRendered);
          }), notifyParent ? switchMap((v) => concat(of(v), onStrategy(null, strategy, (_, work, options) => {
            work(this.cdRef, options.scope);
          }, {
            ngZone: this.patchZone ? this.ngZone : void 0,
            scope: this.cdRef.context || this.cdRef
          }).pipe(ignoreElements()))) : (o$) => o$, this.handleError(), map(() => iterable));
        })
      )
    )), this.handleError());
  }
  handleError() {
    return (o$) => o$.pipe(catchError((err) => {
      this.partiallyFinished = false;
      this.errorHandler.handleError(err);
      return of(null);
    }));
  }
  getDiffer(values) {
    if (this._differ) {
      return this._differ;
    }
    return values ? this._differ = this.iterableDiffers.find(values).create(this._trackBy) : null;
  }
  /** @internal */
  createViewContext(item, computedContext) {
    return new RxVirtualForViewContext(item, this.values, computedContext);
  }
  /** @internal */
  updateViewContext(item, view, computedContext) {
    view.context.updateContext(computedContext);
    view.context.$implicit = item;
    view.context.rxVirtualForOf = this.values;
  }
  static {
    this.ɵfac = function RxVirtualFor_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxVirtualFor)(ɵɵdirectiveInject(TemplateRef));
    };
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _RxVirtualFor,
      selectors: [["", "rxVirtualFor", "", "rxVirtualForOf", ""]],
      inputs: {
        rxVirtualForOf: "rxVirtualForOf",
        rxVirtualForTemplate: "rxVirtualForTemplate",
        strategy: [0, "rxVirtualForStrategy", "strategy"],
        templateCacheSize: [0, "rxVirtualForTemplateCacheSize", "templateCacheSize"],
        renderParent: [0, "rxVirtualForParent", "renderParent"],
        patchZone: [0, "rxVirtualForPatchZone", "patchZone"],
        trackBy: [0, "rxVirtualForTrackBy", "trackBy"],
        renderCallback: [0, "rxVirtualForRenderCallback", "renderCallback"]
      },
      features: [ɵɵProvidersFeature([{
        provide: RxVirtualViewRepeater,
        useExisting: _RxVirtualFor
      }])]
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxVirtualFor, [{
    type: Directive,
    args: [{
      selector: "[rxVirtualFor][rxVirtualForOf]",
      providers: [{
        provide: RxVirtualViewRepeater,
        useExisting: RxVirtualFor
      }],
      standalone: true
    }]
  }], () => [{
    type: TemplateRef
  }], {
    rxVirtualForOf: [{
      type: Input
    }],
    rxVirtualForTemplate: [{
      type: Input
    }],
    strategy: [{
      type: Input,
      args: ["rxVirtualForStrategy"]
    }],
    templateCacheSize: [{
      type: Input,
      args: ["rxVirtualForTemplateCacheSize"]
    }],
    renderParent: [{
      type: Input,
      args: ["rxVirtualForParent"]
    }],
    patchZone: [{
      type: Input,
      args: ["rxVirtualForPatchZone"]
    }],
    trackBy: [{
      type: Input,
      args: ["rxVirtualForTrackBy"]
    }],
    renderCallback: [{
      type: Input,
      args: ["rxVirtualForRenderCallback"]
    }]
  });
})();
var RxVirtualScrollElementDirective = class _RxVirtualScrollElementDirective {
  constructor() {
    this.elementRef = inject(ElementRef);
    this.elementScrolled$ = unpatchedScroll(this.elementRef.nativeElement);
  }
  getElementRef() {
    return this.elementRef;
  }
  measureOffset() {
    return this.elementRef.nativeElement.getBoundingClientRect().top;
  }
  static {
    this.ɵfac = function RxVirtualScrollElementDirective_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxVirtualScrollElementDirective)();
    };
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _RxVirtualScrollElementDirective,
      selectors: [["", "rxVirtualScrollElement", ""]],
      hostAttrs: [1, "rx-virtual-scroll-element"],
      features: [ɵɵProvidersFeature([{
        provide: RxVirtualScrollElement,
        useExisting: _RxVirtualScrollElementDirective
      }])]
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxVirtualScrollElementDirective, [{
    type: Directive,
    args: [{
      selector: "[rxVirtualScrollElement]",
      providers: [{
        provide: RxVirtualScrollElement,
        useExisting: RxVirtualScrollElementDirective
      }],
      host: {
        class: "rx-virtual-scroll-element"
      },
      standalone: true
    }]
  }], null, null);
})();
function observeElementSize(element, config) {
  const extractProp = config?.extract ?? ((entries) => entries[0].contentRect);
  return new Observable((subscriber) => {
    const observer = new ResizeObserver((entries) => {
      subscriber.next(extractProp(entries));
    });
    observer.observe(element, config?.options);
    return () => observer.disconnect();
  });
}
var NG_DEV_MODE = typeof ngDevMode === "undefined" || !!ngDevMode;
var RxVirtualScrollViewportComponent = class _RxVirtualScrollViewportComponent {
  /** @internal */
  constructor() {
    this.elementRef = inject(ElementRef);
    this.scrollStrategy = inject(RxVirtualScrollStrategy, {
      optional: true
    });
    this.scrollElement = inject(RxVirtualScrollElement, {
      optional: true
    });
    this.initialScrollIndex = 0;
    this.elementScrolled$ = this.scrollElement?.elementScrolled$ ?? defer(() => unpatchedScroll(this.runway.nativeElement));
    this._containerRect$ = new ReplaySubject(1);
    this.containerRect$ = this._containerRect$.asObservable();
    this.viewRange = this.scrollStrategy.renderedRange$;
    this.scrolledIndexChange = this.scrollStrategy.scrolledIndex$;
    this.destroy$ = new Subject();
    if (NG_DEV_MODE && !this.scrollStrategy) {
      throw Error("Error: rx-virtual-scroll-viewport requires an `RxVirtualScrollStrategy` to be set.");
    }
    observeElementSize(this.scrollElement?.getElementRef()?.nativeElement ?? this.elementRef.nativeElement, {
      extract: (entries) => ({
        height: Math.round(entries[0].contentRect.height),
        width: Math.round(entries[0].contentRect.width)
      })
    }).pipe(distinctUntilChanged(({
      height: prevHeight,
      width: prevWidth
    }, {
      height,
      width
    }) => prevHeight === height && prevWidth === width), takeUntil(this.destroy$)).subscribe(this._containerRect$);
  }
  ngAfterViewInit() {
    this.scrollStrategy.contentSize$.pipe(distinctUntilChanged(), takeUntil(this.destroy$)).subscribe((size) => {
      this.updateContentSize(size);
    });
    if (this.initialScrollIndex != null && this.initialScrollIndex > 0) {
      this.scrollStrategy.contentSize$.pipe(filter((size) => size > 0), take(1), takeUntil(this.destroy$)).subscribe(() => {
        this.scrollToIndex(this.initialScrollIndex);
      });
    }
  }
  /** @internal */
  ngAfterContentInit() {
    if (ngDevMode && !this.viewRepeater) {
      throw Error("Error: rx-virtual-scroll-viewport requires a `RxVirtualViewRepeater` to be provided.");
    }
    this.scrollStrategy.attach(this, this.viewRepeater);
  }
  /** @internal */
  ngOnDestroy() {
    this.destroy$.next();
    this.scrollStrategy.detach();
  }
  getScrollElement() {
    return this.scrollElement?.getElementRef()?.nativeElement ?? this.runway.nativeElement;
  }
  getScrollTop() {
    return this.getScrollElement().scrollTop;
  }
  scrollTo(position, behavior) {
    this.getScrollElement().scrollTo({
      top: position,
      behavior
    });
  }
  scrollToIndex(index, behavior) {
    this.scrollStrategy.scrollToIndex(index, behavior);
  }
  measureOffset() {
    if (this.scrollElement) {
      const scrollableOffset = this.scrollElement.measureOffset();
      const rect = this.elementRef.nativeElement.getBoundingClientRect();
      return this.getScrollTop() + (rect.top - scrollableOffset);
    } else {
      return 0;
    }
  }
  updateContentSize(size) {
    if (this.scrollElement) {
      this.elementRef.nativeElement.style.height = `${size}px`;
    } else {
      this.scrollSentinel.nativeElement.style.transform = `translate(0, ${size - 1}px)`;
    }
  }
  static {
    this.ɵfac = function RxVirtualScrollViewportComponent_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxVirtualScrollViewportComponent)();
    };
  }
  static {
    this.ɵcmp = ɵɵdefineComponent({
      type: _RxVirtualScrollViewportComponent,
      selectors: [["rx-virtual-scroll-viewport"]],
      contentQueries: function RxVirtualScrollViewportComponent_ContentQueries(rf, ctx, dirIndex) {
        if (rf & 1) {
          ɵɵcontentQuery(dirIndex, RxVirtualViewRepeater, 5);
        }
        if (rf & 2) {
          let _t;
          ɵɵqueryRefresh(_t = ɵɵloadQuery()) && (ctx.viewRepeater = _t.first);
        }
      },
      viewQuery: function RxVirtualScrollViewportComponent_Query(rf, ctx) {
        if (rf & 1) {
          ɵɵviewQuery(_c0, 5)(_c1, 7);
        }
        if (rf & 2) {
          let _t;
          ɵɵqueryRefresh(_t = ɵɵloadQuery()) && (ctx.scrollSentinel = _t.first);
          ɵɵqueryRefresh(_t = ɵɵloadQuery()) && (ctx.runway = _t.first);
        }
      },
      hostAttrs: [1, "rx-virtual-scroll-viewport"],
      inputs: {
        initialScrollIndex: "initialScrollIndex"
      },
      outputs: {
        viewRange: "viewRange",
        scrolledIndexChange: "scrolledIndexChange"
      },
      features: [ɵɵProvidersFeature([{
        provide: RxVirtualScrollViewport,
        useExisting: _RxVirtualScrollViewportComponent
      }])],
      ngContentSelectors: _c2,
      decls: 4,
      vars: 3,
      consts: [["runway", ""], ["sentinel", ""], [1, "rx-virtual-scroll__runway"], [1, "rx-virtual-scroll__sentinel"]],
      template: function RxVirtualScrollViewportComponent_Template(rf, ctx) {
        if (rf & 1) {
          ɵɵprojectionDef();
          ɵɵdomElementStart(0, "div", 2, 0);
          ɵɵconditionalCreate(2, RxVirtualScrollViewportComponent_Conditional_2_Template, 2, 0, "div", 3);
          ɵɵprojection(3);
          ɵɵdomElementEnd();
        }
        if (rf & 2) {
          ɵɵclassProp("rx-virtual-scroll-element", !ctx.scrollElement);
          ɵɵadvance(2);
          ɵɵconditional(!ctx.scrollElement ? 2 : -1);
        }
      },
      styles: [".rx-virtual-scroll-viewport{display:block;width:100%;height:100%;box-sizing:border-box;contain:strict}.rx-virtual-scroll-viewport .rx-virtual-scroll__runway{contain:strict;width:100%;position:absolute;top:0;bottom:0}.rx-virtual-scroll-viewport .rx-virtual-scroll__runway>*{position:absolute}.rx-virtual-scroll-viewport .rx-virtual-scroll__sentinel{width:1px;height:1px;contain:strict;position:absolute;will-change:transform}.rx-virtual-scroll-element{contain:strict;overflow:auto;-webkit-overflow-scrolling:touch}.rx-virtual-scroll-element:not(.rx-virtual-scroll-element:is(.rx-virtual-scroll-element--withSyncScrollbar)){transform:translateZ(0);will-change:scroll-position}\n"],
      encapsulation: 2,
      changeDetection: 0
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxVirtualScrollViewportComponent, [{
    type: Component,
    args: [{
      selector: "rx-virtual-scroll-viewport",
      template: `
    <div
      #runway
      class="rx-virtual-scroll__runway"
      [class.rx-virtual-scroll-element]="!scrollElement"
    >
      @if (!this.scrollElement) {
        <div #sentinel class="rx-virtual-scroll__sentinel"></div>
      }
      <ng-content></ng-content>
    </div>
  `,
      providers: [{
        provide: RxVirtualScrollViewport,
        useExisting: RxVirtualScrollViewportComponent
      }],
      encapsulation: ViewEncapsulation.None,
      host: {
        class: "rx-virtual-scroll-viewport"
      },
      changeDetection: ChangeDetectionStrategy.OnPush,
      imports: [],
      styles: [".rx-virtual-scroll-viewport{display:block;width:100%;height:100%;box-sizing:border-box;contain:strict}.rx-virtual-scroll-viewport .rx-virtual-scroll__runway{contain:strict;width:100%;position:absolute;top:0;bottom:0}.rx-virtual-scroll-viewport .rx-virtual-scroll__runway>*{position:absolute}.rx-virtual-scroll-viewport .rx-virtual-scroll__sentinel{width:1px;height:1px;contain:strict;position:absolute;will-change:transform}.rx-virtual-scroll-element{contain:strict;overflow:auto;-webkit-overflow-scrolling:touch}.rx-virtual-scroll-element:not(.rx-virtual-scroll-element:is(.rx-virtual-scroll-element--withSyncScrollbar)){transform:translateZ(0);will-change:scroll-position}\n"]
    }]
  }], () => [], {
    initialScrollIndex: [{
      type: Input
    }],
    scrollSentinel: [{
      type: ViewChild,
      args: ["sentinel"]
    }],
    runway: [{
      type: ViewChild,
      args: ["runway", {
        static: true
      }]
    }],
    viewRepeater: [{
      type: ContentChild,
      args: [RxVirtualViewRepeater]
    }],
    viewRange: [{
      type: Output
    }],
    scrolledIndexChange: [{
      type: Output
    }]
  });
})();
var RxVirtualScrollWindowDirective = class _RxVirtualScrollWindowDirective {
  constructor() {
    this.document = inject(DOCUMENT);
    this.elementRef = new ElementRef(this.document.documentElement);
    this.elementScrolled$ = unpatchedScroll(this.document);
  }
  getElementRef() {
    return this.elementRef;
  }
  measureOffset() {
    return 0;
  }
  static {
    this.ɵfac = function RxVirtualScrollWindowDirective_Factory(__ngFactoryType__) {
      return new (__ngFactoryType__ || _RxVirtualScrollWindowDirective)();
    };
  }
  static {
    this.ɵdir = ɵɵdefineDirective({
      type: _RxVirtualScrollWindowDirective,
      selectors: [["rx-virtual-scroll-viewport", "scrollWindow", ""]],
      features: [ɵɵProvidersFeature([{
        provide: RxVirtualScrollElement,
        useExisting: _RxVirtualScrollWindowDirective
      }])]
    });
  }
};
(() => {
  (typeof ngDevMode === "undefined" || ngDevMode) && setClassMetadata(RxVirtualScrollWindowDirective, [{
    type: Directive,
    args: [{
      selector: "rx-virtual-scroll-viewport[scrollWindow]",
      providers: [{
        provide: RxVirtualScrollElement,
        useExisting: RxVirtualScrollWindowDirective
      }],
      standalone: true
    }]
  }], null, null);
})();
export {
  AutoSizeVirtualScrollStrategy,
  DynamicSizeVirtualScrollStrategy,
  FixedSizeVirtualScrollStrategy,
  RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS,
  RX_VIRTUAL_SCROLL_DEFAULT_OPTIONS_FACTORY,
  RxVirtualFor,
  RxVirtualForViewContext,
  RxVirtualScrollElement,
  RxVirtualScrollElementDirective,
  RxVirtualScrollStrategy,
  RxVirtualScrollViewport,
  RxVirtualScrollViewportComponent,
  RxVirtualScrollWindowDirective,
  RxVirtualViewRepeater
};
//# sourceMappingURL=@rx-angular_template_virtual-scrolling.js.map
