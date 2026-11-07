// Serves content.pack, the game's files for the web version (models,
// textures, circuits), from the R2 bucket bound to BUCKET: it is larger than
// the 25 MiB a Cloudflare Pages file may have. Each build asks for it with
// its own ?v=<SHA-256>, so it can be kept for a year.
export async function onRequest(context) {
  const object = await context.env.BUCKET.get("content.pack");
  if (!object) {
    return new Response("content.pack not found in R2 bucket", { status: 404 });
  }

  const headers = new Headers();
  object.writeHttpMetadata(headers);
  headers.set("content-type", "application/octet-stream");
  headers.set("content-length", String(object.size));
  headers.set("etag", object.httpEtag);
  headers.set("cache-control", "public, max-age=31536000, immutable");

  return new Response(object.body, { headers });
}
