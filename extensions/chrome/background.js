import {apiBuilder} from "./api.js";

import init from '/tondi-dashboard.js';
(async () => {

    function initPageScript(tabId, args){

        console.log("*** initPageScript", tabId, args);
        console.log("*** location", location);
        // return;

        chrome.scripting.executeScript({
            args: args||[],
            target: {tabId},
            world: "MAIN",
            func: apiBuilder
        });
    }

    //TODO: move to rust
    async function openPopup(){
        if(chrome.action?.openPopup){
            chrome.action.openPopup();
        }else{
            let win = await chrome.windows.getCurrent();
            let width = 400;
            let left = Math.max(0, win.left + win.width - width);
            chrome.windows.create({url:"popup.html", focused:true, left, width, height:600, type:"panel"})
        }
    }

    globalThis.initPageScript = initPageScript;
    globalThis.openPopup = openPopup;

    let tondi_dashboard = await init('/tondi-dashboard_bg.wasm');

    // console.log("init", init);
    // console.log("tondi_dashboard", tondi_dashboard);

    await tondi_dashboard.tondi_dashboard_background();
})();
