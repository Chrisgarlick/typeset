import type { CSSProperties } from "react";

// An A4 page in a client's colours: an illustration of what Typeset makes,
// not a real render (app/typeset.css, .ts-doc).
export function DocPage({
  client,
  kind,
  title,
  site,
  pages,
  colour,
  soft,
  size,
}: {
  client: string;
  kind: string;
  title: string;
  site: string;
  pages: string;
  colour: string;
  soft: string;
  size?: "lg";
}) {
  const style = { "--doc": colour, "--doc-soft": soft } as CSSProperties;
  return (
    <div className="ts-doc" data-size={size} style={style} aria-hidden="true">
      <div className="ts-doc-band">
        <div className="ts-doc-top">
          <span>{client}</span>
          <span>{kind}</span>
        </div>
        <p className="ts-doc-title">{title}</p>
      </div>
      <div className="ts-doc-body">
        <h4>Executive summary</h4>
        <div className="ts-doc-line" />
        <div className="ts-doc-line" />
        <div className="ts-doc-line" style={{ inlineSize: "62%" }} />
        <div className="ts-doc-quote">
          <div>
            <i />
            <i style={{ inlineSize: "55%" }} />
          </div>
        </div>
        <div className="ts-doc-bars">
          {[40, 62, 52, 93, 74].map((h, i) => (
            <i key={i} style={{ blockSize: `${h}%`, opacity: [0.35, 0.55, 0.75, 1, 0.85][i] }} />
          ))}
        </div>
        <div className="ts-doc-foot">
          <span>{site}</span>
          <span>{pages}</span>
        </div>
      </div>
    </div>
  );
}
