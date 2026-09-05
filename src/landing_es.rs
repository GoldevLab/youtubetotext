//! Spanish SEO landings — distinct pages, not thin translations of the English ones.

use crate::family::{Landing, Mode};

pub fn landing_for_es(mode: Mode) -> Landing {
    match mode {
        Mode::Text => TEXT,
        Mode::Audio => AUDIO,
        Mode::Translate => TRANSLATE,
        Mode::Summary => SUMMARY,
        Mode::Srt => SRT,
    }
}

const TEXT: Landing = Landing {
    title: "YouTube a texto — transcripción gratis de un video público | YouTubeForge",
    description: "Pega un enlace de YouTube y obtén la transcripción. Busca, copia o descarga TXT/Markdown. Sin cuenta.",
    eyebrow: "YouTube → texto",
    h1: "Pasa un video de YouTube a texto buscable",
    lead: "Esta página es para leer lo que se dice: notas, citas, buscar un nombre. Usamos las pistas de subtítulos públicas de YouTube. No re-transcribimos el audio en nuestros servidores.",
    howto_title: "Cómo sacar la transcripción",
    howto: [
        ("Pega el enlace", "Watch, Shorts, youtu.be o el id de 11 caracteres. El resultado abre en /?v= con el texto al lado del reproductor."),
        ("Encuentra la línea", "Busca, salta intro/cierre o marca dos líneas. Un clic salta el video."),
        ("Copia o descarga", "Texto plano o Markdown con marcas de tiempo. La URL del resultado no se indexa."),
    ],
    why_title: "Para qué sirve esta herramienta",
    why: [
        ("Leer, no ver", "Una entrevista de una hora es más rápida en texto si solo necesitas una cita."),
        ("Misma sesión", "Audio, traducción, resumen y SRT quedan en el mismo resultado. No vuelves a pegar."),
        ("Editas en el dispositivo", "Corrige un error del auto-subtítulo. Copias y descargas usan lo que ves."),
        ("Sin muro de cookies", "Solo pistas públicas. No somos YouTube ni Google."),
    ],
    examples_title: "Trabajos típicos",
    examples: [
        ("Cita de entrevista", "Busca el nombre, copia tres líneas, pégalas con el minuto."),
        ("Apuntes de clase", "Salta los 45 s de intro y copia el resto en Markdown."),
        ("Revisar accesibilidad", "Lee el auto-subtítulo antes de fiarte; edita un término mal escrito."),
    ],
    limits: "Si YouTube nunca publicó subtítulos (humanos o automáticos), no hay nada que extraer. Videos privados, con restricción de edad o solo en vivo suelen fallar. No inventamos habla a partir del audio.",
    faq: [
        ("¿Es gratis?", "Sí. Sin registro. Puede haber anuncios alrededor; el texto no está de pago."),
        ("¿Hacen speech-to-text?", "No. Cargamos las pistas que YouTube ya tiene. Sin pistas, no hay transcripción."),
        ("¿Dónde queda el resultado?", "En /?v={id}&mode=text. Esas URLs van en noindex para no inundar el buscador."),
        ("¿Hay tope de duración?", "No añadimos uno. Si YouTube tiene pistas, las cargamos."),
        ("¿Puedo guardar tiempos?", "Sí: copia con marcas, Markdown con enlaces, o TXT temporizado."),
    ],
};

const AUDIO: Landing = Landing {
    title: "YouTube a MP3 — audio de un video público | YouTubeForge",
    description: "Descarga el audio de un YouTube público en MP3 (por defecto), M4A, Opus o WAV. Luego abre la transcripción en la misma app.",
    eyebrow: "YouTube → audio",
    h1: "Descarga el audio de un video público de YouTube",
    lead: "Esta página es para la banda sonora, no para el texto. Pedimos un stream solo-audio y lo guardas en MP3 por defecto. Un talk sin subtítulos igual puede tener audio.",
    howto_title: "Cómo guardar el audio",
    howto: [
        ("Pega la URL", "Los mismos enlaces que la transcripción. Pedimos audio adaptativo, no un 1080p muxed."),
        ("Descarga el archivo", "MP3 por defecto. Cambia a M4A, Opus o WAV si lo necesitas."),
        ("Sigue en el mismo video", "Texto, SRT o resumen sin volver a pegar."),
    ],
    why_title: "No es un clon genérico de “YouTube MP3”",
    why: [
        ("Solo audio", "Preferimos streams de audio para no bajar píxeles de video."),
        ("Misma familia", "Después del archivo, un clic a texto o SRT."),
        ("Fallos honestos", "Si YouTube solo da una URL cifrada que no podemos abrir, lo decimos."),
        ("Uso personal", "Solo para contenido que puedes guardar. No somos un archivo pirata."),
    ],
    examples_title: "Cuándo el audio es el trabajo correcto",
    examples: [
        ("Escuchar después", "Guarda la charla para un reproductor que no necesita el video."),
        ("Idioma", "Audio original + transcripción traducida en otra pestaña."),
        ("Sin subtítulos", "Hay habla pero YouTube no publicó timedtext; el audio igual puede salir."),
    ],
    limits: "Solo videos públicos que resolvemos. Streams cifrados, DRM y muros de región/edad fallan. No alojamos una biblioteca de archivos.",
    faq: [
        ("¿Es YouTube a MP3?", "Sí: MP3 es el formato por defecto. También M4A, Opus y WAV. Úsalo solo si puedes guardar una copia personal."),
        ("¿Por qué falló?", "El JSON del reproductor no trajo URL de audio, o el video está bloqueado en esta red."),
        ("¿Qué peso tiene?", "Bitrate × duración (p. ej. ~1 MB/min a 128 kbps). Enviamos el archivo en stream."),
        ("¿Puedo recortar?", "Aún no en la descarga. El recorte está en la transcripción."),
        ("¿Es gratis?", "Sí, con el mismo modelo de anuncios alrededor de la herramienta."),
    ],
};

const TRANSLATE: Landing = Landing {
    title: "Traductor de YouTube — subtítulos con tlang | YouTubeForge",
    description: "Traduce subtítulos de YouTube a otro idioma con el traductor de YouTube (tlang). Las líneas siguen temporizadas.",
    eyebrow: "Subtítulos → otro idioma",
    h1: "Traduce una transcripción de YouTube sin perder los tiempos",
    lead: "No es un traductor de párrafos. Cargamos la pista y pedimos a YouTube que la traduzca (tlang) para que cada línea conserve su inicio. Sirve para subtítulos, estudiar y saltar el reproductor.",
    howto_title: "Cómo funciona aquí la traducción",
    howto: [
        ("Abre el video", "Elige la pista origen (auto inglés vs humana, español, japonés…)."),
        ("Elige Traducir", "El catálogo es el de YouTube. lang y tlang quedan en la URL."),
        ("Exporta si hace falta", "SRT/VTT en las líneas traducidas, o Markdown con tiempos."),
    ],
    why_title: "Por qué no pegar el texto en un chat",
    why: [
        ("Alineación", "Un chat junta frases y pierde minutos. tlang respeta las cues."),
        ("Mismo par de idiomas", "/?v=…&mode=translate&lang=en&tlang=es se puede compartir (noindex)."),
        ("Mismas descargas", "Las cues traducidas salen en SRT, VTT, TXT, Markdown, JSON."),
        ("Si YouTube está saturado", "Apply traduce las líneas que ya están en la página."),
    ],
    examples_title: "Trabajos de traducción",
    examples: [
        ("Estudiar en tu idioma", "Audio en inglés, cues en español, clic para oír el original."),
        ("Archivo bilingüe", "Traduce y descarga SRT."),
        ("Canal en otro idioma", "Auto-subtítulos + tlang. La calidad es la de YouTube."),
    ],
    limits: "Los tiempos se mantienen. Traducimos pistas, no píxeles quemados ni el audio. Nombres y jerga pueden salir mal.",
    faq: [
        ("¿Traduce el video entero?", "Traducimos subtítulos, no hardsubs ni la pista de audio."),
        ("¿Qué idiomas?", "Los que YouTube liste en esa pista. La app muestra el catálogo."),
        ("¿El enlace recuerda el idioma?", "Sí: lang (origen) y tlang (traducción)."),
        ("¿Puedo corregir una mala traducción?", "Modo Edit en tu dispositivo, luego copia/descarga."),
        ("¿Y si YouTube limita las pistas?", "Apply usa el texto ya cargado. No hace falta volver a bajar captions."),
    ],
};

const SUMMARY: Landing = Landing {
    title: "Resumen de YouTube — recap por capítulos + prompt | YouTubeForge",
    description: "Resume un YouTube desde sus subtítulos: recap extractivo por capítulos y un prompt para el modelo que ya pagas. Sin factura extra de IA.",
    eyebrow: "YouTube → resumen",
    h1: "Resume un video desde la transcripción, no desde una caja negra",
    lead: "Muchos “resúmenes IA” no vieron el video. Aquí la fuente es el archivo de subtítulos. Armamos un recap por capítulos con esas frases y copiamos un prompt para el LLM que ya usas.",
    howto_title: "Cómo resumir aquí",
    howto: [
        ("Carga las pistas", "El mismo pegar-enlace. Los capítulos vienen del creador si existen."),
        ("Lee el recap", "Cada capítulo (o todo el talk) toma frases de ese tramo: extractivo, no alucinado."),
        ("Opcional: tu modelo", "Copia el prompt Summary (con la transcripción) a ChatGPT, Claude o uno local."),
    ],
    why_title: "Por qué no un sitio de “AI summary” de un solo tiro",
    why: [
        ("Puedes revisar las cues", "Cada frase del recap es un trozo del texto que tienes debajo."),
        ("Los capítulos importan", "Una clase de 2 horas no es un párrafo. Partimos por capítulos del creador."),
        ("Tú eliges el modelo", "No medimos un API de resumen. Los prompts se copian aquí."),
        ("Sigue en la misma app", "Después: SRT, traducción o audio."),
    ],
    examples_title: "Intenciones distintas de “resumir”",
    examples: [
        ("Charla de conferencia", "Usa capítulos como índice; lee el recap antes de gastar 40 minutos."),
        ("Apuntes", "El prompt Notes (encabezados y glosario) en vez de Summary."),
        ("Cazar citas", "Prompt Quotes y luego clic en los tiempos."),
    ],
    limits: "Basura de subtítulos, basura de recap. No vemos los píxeles: chistes solo visuales no aparecen. Sin capítulos, un solo recap.",
    faq: [
        ("¿Usan GPT en el servidor?", "No. El recap de la página es extractivo. Summary copia un prompt para tu modelo."),
        ("¿Cuánto tarda?", "Bajar pistas suele ser ~1 s. El recap es inmediato después."),
        ("¿Puedo resumir un recorte?", "Recorta las cues y luego copia Summary: usa las líneas visibles."),
        ("¿Y si el ASR está sucio?", "Edita errores claros y luego recap/copia. No limpiamos ASR por ti."),
        ("¿Es gratis?", "Sí. Mismo modelo de anuncios alrededor del flujo."),
    ],
};

const SRT: Landing = Landing {
    title: "YouTube a SRT / VTT — subtítulos desde las pistas | YouTubeForge",
    description: "Descarga subtítulos de YouTube en SRT o VTT para reproductores y editores. Cues con tiempo, pistas e idioma. No es una captura de la transcripción.",
    eyebrow: "YouTube → SRT / VTT",
    h1: "Descarga subtítulos de YouTube en SRT o VTT",
    lead: "Esta página es para un archivo que sueltas en VLC, Premiere o un <track> HTML. Eso no es “copiar el texto”: SRT/VTT necesitan índice, inicio, fin y línea. Los armamos desde las pistas temporizadas.",
    howto_title: "Cómo sacar un SRT",
    howto: [
        ("Pega el video", "Ábrelo para que las cues tengan inicio y duración."),
        ("Elige la pista", "Las humanas suelen ganar en nombres. Traduce antes si el archivo va en otro idioma."),
        ("Descarga SRT o VTT", "SRT usa coma en milisegundos; VTT es WEBVTT con punto. Mismas cues."),
    ],
    why_title: "Por qué una landing de SRT",
    why: [
        ("Los reproductores quieren archivos", "Pegar en un doc no es un subtítulo. SRT sí."),
        ("VTT para la web", "Para <video> HTML5 baja VTT, no TXT."),
        ("Recorta y exporta", "Salta intro o marca dos líneas; el archivo solo lleva ese rango."),
        ("También API", "GET /api/transcript?v=…&fmt=srt para scripts. Con clave, más cuota."),
    ],
    examples_title: "Trabajos de subtítulo",
    examples: [
        ("Reproducción local", "SRT al lado del archivo en VLC."),
        ("Clip de curso", "Recorta la demo y exporta VTT para la lección."),
        ("Archivo traducido", "tlang y luego SRT — subtítulo traducido y temporizado."),
    ],
    limits: "Sin pistas no hay SRT. No hacemos OCR de hardsubs. Los tiempos siguen a YouTube; los autos que se repiten se colapsan.",
    faq: [
        ("¿SRT o VTT?", "SRT es el clásico de editores. VTT es el estándar web. Salen de las mismas cues."),
        ("¿UTF-8?", "Sí. Úsalo en pistas que no son inglés."),
        ("¿JSON?", "Sí: fmt=json en la API o el botón JSON."),
        ("¿Edit cambia el SRT?", "Las descargas de la página usan las cues que ves, con ediciones locales."),
        ("¿Tope de cues?", "Rechazamos ingestos absurdos; un video normal cabe."),
    ],
};
