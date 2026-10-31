// Browsers only offer to install pages with a service worker. The page always needs the board
// server anyway, so requests go straight to the network and nothing is cached.
self.addEventListener("fetch", (event) => event.respondWith(fetch(event.request)));
