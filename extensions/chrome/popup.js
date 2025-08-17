// We disable caching during development so that we always view the latest version.
// if ('serviceWorker' in navigator && window.location.hash !== "#dev") {
//     window.addEventListener('load', function () {
//         navigator.serviceWorker.register('./pwa.js');
//     });
// }

// import init from '/kaspa-egui-57c7a8dd13e092be.js';
// init('/kaspa-egui-57c7a8dd13e092be_bg.wasm');

// -----------

// document.querySelector("#btn").addEventListener("click", ()=>{
//     chrome.runtime.sendMessage({
//         target: "offscreen",
//         data: "message from popup"
//     }, (msg)=>{
//         if (msg){
//             alert("msg:"+msg);
//         }
//     })
// })

// chrome.runtime.onMessage.addListener((message, sender, reply)=>{
//     console.log("popup:", message)
//     let {data} = message;
//     if (data?.counter){
//         document.querySelector("#counter").textContent = `counter: ${data.counter}`;
//     }
// })

// import init from '/tondi-dashboard.js';
// let tondi_dashboard = init('/tondi-egui_bg.wasm');
import init from '/tondi-dashboard.js';
(async () => {
    let tondi_dashboard = await init('/tondi-dashboard_bg.wasm');

    // const wasm = await tondi.default('./tondi-wallet/tondi-wallet_bg.wasm');
    await tondi_dashboard.tondi_dashboard_main();
})();


// wasm_bindgen('/kaspa-ng_bg.wasm');