// Browsers only offer to install pages with a service worker. The page always needs the board
// server anyway, so requests go straight to the network and nothing is cached. The board's event
// stream skips the worker entirely so it isn't held open through it.
self.addEventListener("fetch", (event) => {
  if (new URL(event.request.url).pathname === "/events") {
    return;
  }
  event.respondWith(fetch(event.request));
});
