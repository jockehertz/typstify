#set page(paper: "a4")

#let content = inputs.v

#for elem in content.enumerate() [
  $
  #elem \
  $
]
