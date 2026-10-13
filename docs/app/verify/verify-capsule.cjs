const {chromium}=require('/Users/brent.jacobs/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const assert=require('node:assert/strict');
const url='http://localhost:58329/?key=b736111000a48915a143b1af90bd6bc3f888d9d171a24b54b1f1e59fc717ca3c';
(async()=>{
const browser=await chromium.launch({executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',headless:true,args:['--disable-gpu']});
try{
const page=await browser.newPage({viewport:{width:1440,height:950}});
const errors=[];page.on('pageerror',e=>errors.push(e.message));
await page.goto(url,{waitUntil:'domcontentloaded'});
await page.waitForFunction(()=>window.capsuleStudy);
await page.screenshot({path:'/private/tmp/capsule-separated.png'});
await page.click('#a [data-page="1"]');await page.click('#b [data-page="1"]');await page.click('#a .run-button');
await page.click('#join');await page.waitForFunction(()=>capsuleStudy.model.joined&&!capsuleStudy.model.busy);
assert.deepEqual(await page.evaluate(()=>[capsuleStudy.model.a.page,capsuleStudy.model.b.page,capsuleStudy.model.a.running]),[1,1,false]);
await page.screenshot({path:'/private/tmp/capsule-joined.png'});
await page.click('#a .drawer-button');await page.click('#insert');await page.waitForTimeout(500);
await page.screenshot({path:'/private/tmp/capsule-drawer.png'});
assert(await page.evaluate(()=>capsuleStudy.model.docked));
await page.click('#a .drawer-button');assert(await page.evaluate(()=>capsuleStudy.model.docked&&!capsuleStudy.model.drawer));
await page.click('#a .drawer-button');await page.waitForTimeout(500);
await page.click('#separate');await page.waitForFunction(()=>!capsuleStudy.model.joined&&!capsuleStudy.model.busy);
assert.deepEqual(await page.evaluate(()=>[capsuleStudy.model.a.page,capsuleStudy.model.b.page,capsuleStudy.model.a.running,capsuleStudy.model.docked,capsuleStudy.model.drawer]),[1,1,false,true,true]);
await page.click('#design');await page.uncheck('#allowJoin');assert(await page.locator('#join').isDisabled());
await page.check('#allowJoin');await page.click('#join');await page.waitForFunction(()=>capsuleStudy.model.joined&&!capsuleStudy.model.busy);
await page.screenshot({path:'/private/tmp/capsule-design.png'});
const before=await page.evaluate(()=>({a:{...capsuleStudy.model.a},b:{...capsuleStudy.model.b}}));
let box=await page.locator('#b .grip').boundingBox();await page.mouse.move(box.x+100,box.y+10);await page.mouse.down();await page.mouse.move(box.x+140,box.y+40,{steps:8});await page.mouse.up();
const after=await page.evaluate(()=>({a:{...capsuleStudy.model.a},b:{...capsuleStudy.model.b}}));assert(Math.abs(after.a.x-before.a.x-40)<1);assert.equal(after.a.x-before.a.x,after.b.x-before.b.x);
await page.click('#reset');await page.click('#join');await page.waitForTimeout(200);await page.click('#reset');await page.waitForTimeout(1400);assert(await page.evaluate(()=>!capsuleStudy.model.joined&&!capsuleStudy.model.busy&&capsuleStudy.model.cap===0));
await page.emulateMedia({reducedMotion:'reduce'});await page.click('#join');await page.waitForFunction(()=>capsuleStudy.model.joined&&!capsuleStudy.model.busy);await page.click('#reset');
box=await page.locator('#b .grip').boundingBox();await page.mouse.move(box.x+100,box.y+10);await page.mouse.down();await page.mouse.move(box.x-40,box.y+10,{steps:12});await page.mouse.up();await page.waitForFunction(()=>capsuleStudy.model.joined&&!capsuleStudy.model.busy);
await page.click('#a .drawer-button');
let meter=await page.locator('#meter .meter-grip').boundingBox();let bay=await page.locator('#bay').boundingBox();await page.mouse.move(meter.x+80,meter.y+9);await page.mouse.down();await page.mouse.move(bay.x+95,bay.y+10,{steps:12});await page.mouse.up();assert(await page.evaluate(()=>capsuleStudy.model.docked));
await page.click('#eject');assert(await page.evaluate(()=>!capsuleStudy.model.docked));
await page.setViewportSize({width:760,height:900});await page.screenshot({path:'/private/tmp/capsule-narrow.png'});
assert.deepEqual(errors,[]);console.log('PASS: display state, join/separate, drawer containment, design socket rules, grouped drag, interrupted animation, reduced motion, drag-to-join, drag-to-insert, eject, narrow viewport; no JS errors.');
}finally{await browser.close()}
})().catch(e=>{console.error(e);process.exit(1)});
