import{invoke as i}from"@tauri-apps/api/tauri";class t{constructor(){}open_window(n){return i("plugin:windows|open_window",{configuration:n})}open_empty_window(n,o){return i("plugin:windows|open_empty_window",{configuration:n,options:o})}open_existing_window(n,o){return i("plugin:windows|open_existing_window",{configuration:n,windowToUse:o})}send_to_focused(n,o){return i("plugin:windows|send_to_focused",{channel:n,payload:o})}send_to_all(n,o){return i("plugin:windows|send_to_all",{channel:n,payload:o})}get_focused_window(){return i("plugin:windows|get_focused_window")}get_last_active_window(){return i("plugin:windows|get_last_active_window")}}const e=new t;Object.freeze(e);var _=e;export{_ as default};
