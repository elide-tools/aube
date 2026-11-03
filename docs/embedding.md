
## Studio request correlation

When `ELIDE_STUDIO_SESSION_ID` contains a bounded ASCII session ID, registry and
package-download requests to `https://npm.pkg.st` include
`x-elide-studio-session-id`. Other origins receive no session header. Requests
originating at Pkgst cannot follow redirects outside that HTTPS origin. The ID
is correlation metadata only: it does not alter package integrity, authentication,
or content-addressed cache identity.
