import {HttpClient} from '@angular/common/http';
import {inject, Injectable} from '@angular/core';
import {MatSnackBar} from '@angular/material/snack-bar';
import {catchError, tap, throwError} from 'rxjs';
import {UserSettingsRdo} from './models/user-settings.rdo';
import {UserSettingsDto} from './models/user-settings.dto';
import {parseErrorMessage} from '@shared/error-utils';

@Injectable({providedIn: 'root'})
export class SettingsService {
    private readonly _http = inject(HttpClient);
    private readonly _snack = inject(MatSnackBar);

    public getSettings() {
        return this._http.get<UserSettingsRdo>('/settings');
    }

    public updateSettings(userSettings: UserSettingsDto) {
        return this._http.post('/settings', userSettings).pipe(
            tap(() => this._snack.open('Settings updated', 'OK')),
            catchError((error) => {
                this._snack.open(parseErrorMessage(error, 'Failed to update settings'), 'Close');
                return throwError(() => error);
            })
        );
    }

    public testSoundCloudUrl(url: string) {
        return this._http.post<string>('/settings/test/soundcloud', {url}).pipe(
            tap(() => this._snack.open('SoundCloud URL is valid', 'OK')),
            catchError((error) => {
                this._snack.open(parseErrorMessage(error, 'Invalid configuration'), 'Close');
                return throwError(() => error);
            })
        );
    }

    public testProxy(proxyUrl: string) {
        return this._http.post<string>('/settings/test/proxy', {proxyUrl}).pipe(
            tap(() => this._snack.open('Proxy connection successful', 'OK')),
            catchError((error) => {
                this._snack.open(parseErrorMessage(error, 'Proxy verification failed'), 'Close');
                return throwError(() => error);
            })
        );
    }

    public getAdbDevices() {
        return this._http.get<string[]>('/devices');
    }
}

@Injectable({providedIn: 'root'})
export class SettingsTestingService {

}