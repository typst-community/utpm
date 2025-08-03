#import "@preview/chic-hdr:0.5.0": *
#import "@preview/rose-pine:0.2.0": apply
#import "@preview/codly:1.3.0": *

#let project = (name: "", description: [], sidenote: [], black: false, cols: 1, dark: false, doc) => {
  show: it => {
    if black {
      it
    } else {
      apply(variant: if dark { "rose-pine" } else { "rose-pine-dawn" })(it)
    }
  }

  show: codly-init.with()
  codly();

  align(
    center + horizon,
    text(size: 35pt, heading(level: 1, outlined: false)[#name]),
  )
  align(center + horizon, text(size: 20pt, description))
  sidenote
  pagebreak()
  align(
    center,
  )[
    #set outline.entry(fill: line(length: 100%, stroke: (dash: "dotted")))
    #outline(title: "Sommaire")
  ]
  pagebreak()

  show: chic.with(
    chic-footer(center-side: chic-page-number()),
    chic-header(left-side: emph(chic-heading-name()), right-side: smallcaps(name)),
    chic-separator(1pt),
    chic-offset(7pt),
    chic-height(1.5cm),
  )
  set heading(numbering: "I.1.A.a - ")
  show heading: set text(size: 12pt)
  set terms(separator: ": ")

  if cols > 1 {
    columns(cols)[#doc]
  } else {
    doc
  }
}