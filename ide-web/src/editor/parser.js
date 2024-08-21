// This file was generated by lezer-generator. You probably shouldn't edit it.
import {LRParser} from "@lezer/lr"
const spec_Identifier = {__proto__:null,if:70, IF:70, true:72, false:72, Void:74, Int:74, String:74, Boolean:74, not:76, NOT:76, then:78, THEN:78, else:80, ELSE:80, end:82, END:82, function:84, FUNCTION:84, output:90, OUTPUT:90, return:92, RETURN:92}
export const parser = LRParser.deserialize({
  version: 14,
  states: "(vQYQPOOOOQO'#Ca'#CaOOQO'#Cd'#CdOOQO'#Ch'#ChOOQO'#Ci'#CiOOQO'#Cc'#CcO}QPO'#C`OOQO'#Co'#CoO!iQPO'#CnOOQO'#Cu'#CuO!nQPO'#CtOOQO'#Cw'#CwO!nQPO'#CvOOQO'#Cx'#CxOOQO'#C_'#C_OOQO'#Cy'#CyQYQPOOOOQO'#Cz'#CzO}QPO,58zOOQO'#Cj'#CjO#VQPO,58zO#aQPO,59YOOQO,59`,59`OOQO,59b,59bOOQO-E6w-E6wOOQO-E6x-E6xO#VQPO1G.fO#fQPO'#CkO#pQPO1G.fO#xQPO'#CrO#YQPO1G.tO#pQPO7+$QOOQO'#Cl'#ClOOQO'#Cm'#CmO$QQPO7+$QO#YQPO7+$QO$VQPO'#CsOOQO,59^,59^O$[QPO,59^O#sQPO7+$`O$QQPO<<GlO#YQPO<<GlOOQO<<Gl<<GlO#sQPO<<GlO$dQPO,59_O$iQPO'#C{O$nQPO1G.xOOQO1G.x1G.xO$vQPO<<GzOOQOAN=WAN=WO#sQPOAN=WO$QQPOAN=WOOQO1G.y1G.yOOQO,59g,59gOOQO-E6y-E6yOOQO7+$d7+$dOOQOAN=fAN=fO$QQPOG22rOOQOG22rG22rOOQOLD(^LD(^",
  stateData: "${~OrOSPOS~OUTOXTOYTOZTOsPOtQOuROvSOzVO}XO!OZO~OUTOXTOYTOZTOtQOuROvSOwcO~OUeO~OUTOXTOYTOZTOtQOuROvSO~Ox_Py_P~PYOemO~Ox_Xy_X~PYOxpOyqO~OUtOduO~OsPO~O{|O~Od!PO|}O~OuRO~OUtO~Od!XO|}O~OzVO~O",
  goto: "&UpPPPq{!VP!m#RPPP#a#R#r#x$X$_{$nPP${%O{%U{%`{%j%x&Oa_O`djknsya^O`djknsy`UO`djknsyQzrQ!RxQ![!TR!]!Z`]O`djknsySaUbQfYRg[iTOUY[`bdjknsyhTOUY[`bdjknsyR!U|QdURjbQldQojQwnQ{sR!SyQslRyoQrlQxoQ!QwQ!T{R!Z!S`WO`djknsyR!Y!QRneQvmR!V}aYO`djknsya[O`djknsyQ`OSh`kZkdjnsyQbURibQ!OvR!W!O",
  nodeNames: "⚠ LineComment Program Atom IfStatement IfKeyword Identifier Expression Boolean String Number Operator TypeAnnotation NotKeyword ThenKeyword Block ElseKeyword EndKeyword FunctionDeclaration FunctionKeyword ) ( ParameterList Parameter OutputStatement OutputKeyword ReturnStatement ReturnKeyword ExpressionStatement",
  maxTerm: 46,
  nodeProps: [
    ["openedBy", 20,"("],
    ["closedBy", 21,")"]
  ],
  skippedNodes: [0,1],
  repeatNodeCount: 3,
  tokenData: "%}~RcXY!^YZ!^]^!^pq!^rs!ost$]xy$tyz$yz{%O{|%O|}%T}!O%O!P!Q%O!Q![%Y![!]%b!_!`%g!c!}%o#R#S%o#T#o%o~!cSr~XY!^YZ!^]^!^pq!^~!rVOr!ors#Xs#O!o#O#P#^#P;'S!o;'S;=`$V<%lO!o~#^OX~~#aRO;'S!o;'S;=`#j;=`O!o~#mWOr!ors#Xs#O!o#O#P#^#P;'S!o;'S;=`$V;=`<%l!o<%lO!o~$YP;=`<%l!o~$bSP~OY$]Z;'S$];'S;=`$n<%lO$]~$qP;=`<%l$]~$yOe~~%OOd~~%TOZ~~%YO|~~%_PY~!Q![%Y~%gO{~~%lPZ~!_!`%O~%tRU~!c!}%o#R#S%o#T#o%o",
  tokenizers: [0],
  topRules: {"Program":[0,2]},
  specialized: [{term: 6, get: (value) => spec_Identifier[value] || -1}],
  tokenPrec: 0
})
