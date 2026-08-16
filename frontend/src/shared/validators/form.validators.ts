import {pattern, SchemaPath} from '@angular/forms/signals';

export const soundCloudUrl = (path: SchemaPath<string>) => {
    pattern(path, /^https?:\/\/(www\.)?soundcloud\.com\/[a-z0-9\-_]+(\/.*)?$/i, {
        message: 'Invalid SoundCloud URL'
    });
};

export const youTubeUrl = (path: SchemaPath<string>) => {
    pattern(path, /^https?:\/\/(www\.|music\.|m\.)?(youtube\.com|youtu\.be)\/.*$/i, {
        message: 'Invalid YouTube URL'
    });
};

export const downloadUrl = (path: SchemaPath<string>) => {
    pattern(path, /^https?:\/\/(www\.|music\.|m\.)?(soundcloud\.com|youtube\.com|youtu\.be)\/.*$/i, {
        message: 'Invalid SoundCloud or YouTube URL'
    });
};

export const englishOnly = (path: SchemaPath<string>) => {
    pattern(path, /^[a-zA-Z0-9\s\-_.:/]*$/, {
        message: 'Only English letters, numbers and basic symbols are allowed'
    });
};