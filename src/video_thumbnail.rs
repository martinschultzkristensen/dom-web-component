//src/video_thumbnail.rs
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{CanvasRenderingContext2d, Event, File, HtmlCanvasElement, HtmlVideoElement, Url};
use yew::prelude::Callback;

// Draws the first frame of `file` onto an off-screen canvas and reports the
// resulting base64 PNG via `on_thumbnail`. Same base64-data-URL convention
// used throughout the app for image uploads.
pub fn extract_video_thumbnail(file: File, on_thumbnail: Callback<String>) {
    let Ok(object_url) = Url::create_object_url_with_blob(&file) else {
        return;
    };
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let Some(video) = document
        .create_element("video")
        .ok()
        .and_then(|el| el.dyn_into::<HtmlVideoElement>().ok())
    else {
        return;
    };

    video.set_src(&object_url);
    video.set_muted(true);
    let _ = video.style().set_property("display", "none");
    let _ = body.append_child(&video);

    let video_for_capture = video.clone();
    let object_url_for_cleanup = object_url.clone();

    let onloadeddata = Closure::wrap(Box::new(move |_event: Event| {
        if let Some(canvas) = document
            .create_element("canvas")
            .ok()
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            canvas.set_width(video_for_capture.video_width());
            canvas.set_height(video_for_capture.video_height());

            if let Some(ctx) = canvas
                .get_context("2d")
                .ok()
                .flatten()
                .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
            {
                if ctx
                    .draw_image_with_html_video_element(&video_for_capture, 0.0, 0.0)
                    .is_ok()
                {
                    if let Ok(data_url) = canvas.to_data_url() {
                        on_thumbnail.emit(data_url);
                    }
                }
            }
        }

        video_for_capture.remove();
        let _ = Url::revoke_object_url(&object_url_for_cleanup);
    }) as Box<dyn FnMut(Event)>);

    video.set_onloadeddata(Some(onloadeddata.as_ref().unchecked_ref()));
    onloadeddata.forget();
}
