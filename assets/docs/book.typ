
#import "@preview/shiroa:0.2.3": *

#show: book


#book-meta(
  title: "UTPM Docs",
  language: "en",
  authors: ("Thomas Quemin <Thumuss> & typst-community",),
  description: "",
  summary: [
    - #prefix-chapter("index.typ")[Introduction]
    = Usage
    - #chapter(none)[Packages]
      - #chapter("usage/packages/bulk_delete.typ")[Bulk delete]
    = Details
    - #chapter("details/description.typ")[Description]
  ]
)


// re-export page template
#import "/templates/page.typ": project
#let book-page = project
