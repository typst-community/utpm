#import "/book.typ": book-page

#show: book-page.with(title: "Introduction")
#set terms(separator: ": ")
#let separateur = align(center, line(length: 60%, stroke: (paint: white))) // Marche pas pour le moment 
