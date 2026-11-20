#let green = rgb("#234632")
#let ink = rgb("#23352a")
#let muted = rgb("#4d6053")
#let body-fonts = ("Libertinus Serif", "Noto Sans Arabic", "Noto Sans Devanagari", "Noto Sans Bengali", "Noto Sans SC", "Noto Emoji")
#let display-fonts = ("Space Grotesk", ..body-fonts)

#set document(title: invitation.title, author: "Frizzi")
#set page("a4", margin: (x: 21mm, top: 18mm, bottom: 20mm))
#set text(lang: invitation.lang, font: body-fonts, size: 12pt, fill: ink)
#set par(leading: 0.56em)

#let heading-label(label) = text(font: display-fonts, size: 10pt, weight: "bold", fill: green, label)
#let card(label, fill, content) = block(
  width: 100%, fill: fill, radius: 9pt, inset: 13pt, breakable: false,
  [#heading-label(label) #v(5pt) #content],
)
#let action(url, label) = link(url, underline(text(font: display-fonts, size: 10pt, fill: green, weight: "bold", label)))

#block(
  width: 100%, fill: rgb("#eef6e4"), inset: (x: 15pt, y: 10pt),
  radius: 11pt, stroke: (top: 3pt + rgb("#9ed169")),
  grid(
    columns: (1fr, 50mm), gutter: 5mm, align: (start + horizon, end + horizon),
    [
      #text(dir: ltr, font: display-fonts, size: 23pt, weight: "bold", fill: green)[frizzi#text(fill: rgb("#639539"))[.]]
      #linebreak()
      #heading-label(invitation.labels.invitation)
    ],
    pdf.artifact(image("/src/mail/invitation-guests.webp", width: 50mm)),
  ),
)

#v(16pt)
#heading(level: 1, outlined: false, text(font: display-fonts, size: 32pt, weight: "bold", fill: green, invitation.title))
#v(3pt)
#text(size: 15pt, weight: "bold", invitation.headline)
#linebreak()
#text(invitation.intro)
#v(12pt)

#card(invitation.labels.when, rgb("#eef6e4"))[
  #text(font: display-fonts, size: 14pt, weight: "bold", invitation.dates)
  #linebreak()
  #text(size: 10pt, fill: muted)[#invitation.labels.timezone: #text(dir: ltr, invitation.timezone)]
]
#card(invitation.labels.where, rgb("#fff1dc"))[
  #text(font: display-fonts, size: 14pt, weight: "bold", invitation.location)
  #v(3pt)
  #action(invitation.directions, invitation.labels.directions)
  #if invitation.website != none {
    h(1em)
    action(invitation.website, invitation.labels.website)
    linebreak()
    text(size: 8pt, fill: muted, "Geoapify · © OpenStreetMap contributors")
  }
]
#v(9pt)
#text(font: display-fonts, size: 10.5pt, fill: muted)[#invitation.labels.organizer #strong(invitation.organizer)]
#if invitation.description != none and invitation.description.trim() != "" [
  #v(13pt)
  #line(length: 100%, stroke: 0.6pt + rgb("#dbe5d8"))
  #heading-label(invitation.labels.description)
  #v(3pt)
  #invitation.description.split("\n").map(text).join(linebreak())
]
