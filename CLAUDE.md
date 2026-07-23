# Project notes

dom-web-component (yew-web-app) is a pure web app — NOT a Tauri desktop app.
Do not use Tauri invoke/IPC, window.__TAURI__, or media/-path resolution here.
(danceOmatic is a separate Tauri desktop project, used only as a visual/layout reference.)

## choreography_page.rs plan
- ChoreographyEntry { number, video_thumbnail: Option<String>, title, duration }
- ChoreographyPage holds two Vecs: draft_choreographies, confirmed_choreographies
- "+ tilføj dans" pushes a new empty entry to draft_choreographies
- VideoList (molecules/video_list.rs) is presentational: renders NR., dropzone/thumbnail,
  Title/Længde inputs, "checkout dance {number}" button — dispatches callbacks by `number`
- checkout moves the entry from draft_choreographies to confirmed_choreographies (local state only)
- Thumbnail: file input -> object URL -> off-screen <video> -> canvas.draw_image -> base64 PNG,
  then revoke the object URL. Same base64-data-URL convention as DancerPage's image upload.