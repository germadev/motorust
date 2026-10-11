export async function onRequest(context) {
  const file = context.params.file;
  if (!file) {
    return new Response("Not found", { status: 404 });
  }

  // 1. Intentar servir desde el bucket R2 vinculado a BUCKET
  if (context.env && context.env.BUCKET) {
    try {
      let object = await context.env.BUCKET.get(`downloads/${file}`);
      if (!object) {
        object = await context.env.BUCKET.get(file);
      }

      if (object) {
        const headers = new Headers();
        object.writeHttpMetadata(headers);
        headers.set("content-disposition", `attachment; filename="${file}"`);
        headers.set("content-length", String(object.size));
        headers.set("etag", object.httpEtag);
        headers.set("cache-control", "public, max-age=31536000, immutable");

        if (file.endsWith(".zip")) {
          headers.set("content-type", "application/zip");
        } else if (file.endsWith(".tar.gz")) {
          headers.set("content-type", "application/gzip");
        } else {
          headers.set("content-type", "application/octet-stream");
        }

        return new Response(object.body, { headers });
      }
    } catch (err) {
      console.error("Error al obtener archivo de R2:", err);
    }
  }

  // 2. Fallback: Redirigir al asset correspondiente en GitHub Releases
  const tagMatch = file.match(/motorust-(\d+\.\d+\.\d+)-/);
  const tag = tagMatch ? `v${tagMatch[1]}` : "latest";
  const ghUrl = tag === "latest"
    ? `https://github.com/germadev/motorust/releases/latest/download/${file}`
    : `https://github.com/germadev/motorust/releases/download/${tag}/${file}`;

  return Response.redirect(ghUrl, 302);
}
