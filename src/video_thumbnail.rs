use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, Event, File, HtmlCanvasElement, HtmlVideoElement, Url,
};
use yew::Callback;

const THUMBNAIL_TIME_SECONDS: f64 = 5.0;

pub fn extract_video_thumbnail(file: File, callback: Callback<String>) {
    let Ok(video_url) = Url::create_object_url_with_blob(&file) else {
        return;
    };

    let Some(window) = web_sys::window() else {
        let _ = Url::revoke_object_url(&video_url);
        return;
    };

    let Some(document) = window.document() else {
        let _ = Url::revoke_object_url(&video_url);
        return;
    };

    let Ok(video_element) = document.create_element("video") else {
        let _ = Url::revoke_object_url(&video_url);
        return;
    };

    let Ok(video) = video_element.dyn_into::<HtmlVideoElement>() else {
        let _ = Url::revoke_object_url(&video_url);
        return;
    };

    video.set_src(&video_url);
    video.set_muted(true);
    video.set_preload("metadata");
    let _ = video.set_attribute("playsinline", "true");

    let video_for_seeked = video.clone();
    let video_url_for_seeked = video_url.clone();
    let document_for_seeked = document.clone();
    let callback_for_seeked = callback.clone();

    let on_seeked = Closure::wrap(Box::new(move |_event: Event| {
        let width = if video_for_seeked.video_width() > 0 {
            video_for_seeked.video_width()
        } else {
            320
        };

        let height = if video_for_seeked.video_height() > 0 {
            video_for_seeked.video_height()
        } else {
            180
        };

        let Ok(canvas_element) = document_for_seeked.create_element("canvas") else {
            let _ = Url::revoke_object_url(&video_url_for_seeked);
            return;
        };

        let Ok(canvas) = canvas_element.dyn_into::<HtmlCanvasElement>() else {
            let _ = Url::revoke_object_url(&video_url_for_seeked);
            return;
        };

        canvas.set_width(width);
        canvas.set_height(height);

        let Ok(Some(context_value)) = canvas.get_context("2d") else {
            let _ = Url::revoke_object_url(&video_url_for_seeked);
            return;
        };

        let Ok(context) = context_value.dyn_into::<CanvasRenderingContext2d>() else {
            let _ = Url::revoke_object_url(&video_url_for_seeked);
            return;
        };

        let _ = context.draw_image_with_html_video_element(&video_for_seeked, 0.0, 0.0);

        if let Ok(data_url) = canvas.to_data_url_with_type("image/jpeg") {
            callback_for_seeked.emit(data_url);
        }

        let _ = Url::revoke_object_url(&video_url_for_seeked);
    }) as Box<dyn FnMut(Event)>);

    video.set_onseeked(Some(on_seeked.as_ref().unchecked_ref()));
    on_seeked.forget();

    let video_for_metadata = video.clone();

    let on_loaded_metadata = Closure::wrap(Box::new(move |_event: Event| {
        let duration = video_for_metadata.duration();

        let thumbnail_time = if duration.is_finite() && duration > THUMBNAIL_TIME_SECONDS + 0.5 {
            THUMBNAIL_TIME_SECONDS
        } else if duration.is_finite() && duration > 0.5 {
            duration / 2.0
        } else {
            0.1
        };

        video_for_metadata.set_current_time(thumbnail_time);
    }) as Box<dyn FnMut(Event)>);

    video.set_onloadedmetadata(Some(on_loaded_metadata.as_ref().unchecked_ref()));
    on_loaded_metadata.forget();
}