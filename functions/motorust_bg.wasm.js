export async function onRequest(context) {
  // Obtiene el archivo .wasm desde el bucket R2 vinculado a la variable BUCKET
  const object = await context.env.BUCKET.get("motorust_bg.wasm");
  if (!object) {
    return new Response("motorust_bg.wasm not found in R2 bucket", { status: 404 });
  }

  const headers = new Headers();
  object.writeHttpMetadata(headers);
  headers.set("content-type", "application/wasm");
  headers.set("content-length", String(object.size));
  headers.set("etag", object.httpEtag);
  // Cachear en el CDN y navegador para descargas ultrarrápidas
  headers.set("cache-control", "public, max-age=31536000, immutable");

  return new Response(object.body, { headers });
}
