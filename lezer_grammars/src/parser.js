// This file was generated by lezer-generator. You probably shouldn't edit it.
import {LRParser} from "@lezer/lr"
const spec_Identifier = {__proto__:null,if:78, IF:78, true:80, false:80, Void:82, Int:82, String:82, Boolean:82, not:84, NOT:84, then:86, THEN:86, else:88, ELSE:88, end:90, END:90, function:92, FUNCTION:92, output:98, OUTPUT:98, return:100, RETURN:100}
export const parser = LRParser.deserialize({
  version: 14,
  states: ")YQYQPOOOOQO'#Ca'#CaO#RQPO'#CgOOQO'#Ch'#ChOOQO'#Cl'#ClOOQO'#Cm'#CmOOQO'#Cc'#CcO#YQPO'#C`OOQO'#Cs'#CsO#tQPO'#CrOOQO'#Cw'#CwO#yQPO'#CvOOQO'#Cy'#CyO#yQPO'#CxOOQO'#C|'#C|OOQO'#C_'#C_OOQO'#C}'#C}QYQPOOO$bQPO,59PO#yQPO,59fO$iQPO'#CgOOQO'#DO'#DOO#YQPO,58zOOQO'#Cn'#CnO$pQPO,58zO$zQPO,59^OOQO,59b,59bOOQO,59d,59dOOQO-E6{-E6{OOQO1G.k1G.kO%PQPO1G.kOOQO1G/Q1G/QOOQO-E6|-E6|O$pQPO1G.fO%WQPO'#CoO%bQPO1G.fO%jQPO'#CtO$sQPO1G.xOOQO7+$V7+$VO%bQPO7+$QOOQO'#Cp'#CpOOQO'#Cq'#CqOOQO7+$Q7+$QO$sQPO7+$QO%rQPO'#CuOOQO,59`,59`O%wQPO,59`O%eQPO7+$dOOQO<<Gl<<GlO$sQPO<<GlO%eQPO<<GlO&PQPO,59aO&UQPO'#DPO&ZQPO1G.zOOQO1G.z1G.zOOQO<<HO<<HOO%eQPOAN=WOOQOAN=WAN=WOOQO1G.{1G.{OOQO,59k,59kOOQO-E6}-E6}OOQO7+$f7+$fOOQOG22rG22r",
  stateData: "&c~OvOSPOS~OUQO]UO^UO_UOwPOxROySOzTO!OWO!RYO!S[O~OYbOUZX]ZX^ZX_ZXtZXwZXxZXyZXzZX!OZX!RZX!SZXWZX|ZX}ZX~OocO~P}OUdO]UO^UO_UOxROySOzTO{gO~OUiO~OUdO]UO^UO_UOxROySOzTO~OWmO~PYO{ZX~P}O|cP}cP~PYOYtO~OWvO~PYO|cX}cX~PYO|xO}yO~OU|OW}O~O!P!TO~OW!WO!Q!UO~OySO~OU|O~OW!_O!Q!UO~O",
  goto: "&ftPPPu!R!_P!kP#UP#U#UPPP#g#U#{$R$b$h!R$w%T%W!R%^!R%j!RP!R%v&Y&`e`Oabhnqru{!Re_Oabhnqru{!ReVOabhnqru{!Rd^Oabhnqru{!RSeVfQjZQk]RocoUOVZ]abcfhnqru{!RnUOVZ]abcfhnqru{!RR![!TQhVRqfQshQwqQ!PuQ!S{R!Y!RQ{sR!RwQzsQ!QwQ!X!PQ!Z!SR!`!YeXOabhnqru{!RRuiQ!OtR!]!UeZOabhnqru{!Re]Oabhnqru{!RQaOUlanrQnbZrhqu{!RQfVRpfQ!V!OR!^!V",
  nodeNames: "⚠ LineComment Program Atom IfStatement IfKeyword Identifier Expression ) CallExpression ( ReferenceEpxression Boolean String Number MiscOperator TypeAnnotation NotKeyword ThenKeyword Block ElseKeyword EndKeyword FunctionDeclaration FunctionKeyword ParameterList Parameter OutputStatement OutputKeyword ReturnStatement ReturnKeyword VariableAssignment AssignmentOperator ExpressionStatement",
  maxTerm: 50,
  nodeProps: [
    ["openedBy", 8,"("],
    ["closedBy", 10,")"]
  ],
  skippedNodes: [0,1],
  repeatNodeCount: 3,
  tokenData: "%}~RcXY!^YZ!^]^!^pq!^rs!ost$]xy$tyz$yz{%O{|%O|}%T}!O%O!P!Q%O!Q![%Y![!]%b!_!`%g!c!}%o#R#S%o#T#o%o~!cSv~XY!^YZ!^]^!^pq!^~!rVOr!ors#Xs#O!o#O#P#^#P;'S!o;'S;=`$V<%lO!o~#^O]~~#aRO;'S!o;'S;=`#j;=`O!o~#mWOr!ors#Xs#O!o#O#P#^#P;'S!o;'S;=`$V;=`<%l!o<%lO!o~$YP;=`<%l!o~$bSP~OY$]Z;'S$];'S;=`$n<%lO$]~$qP;=`<%l$]~$yOY~~%OOW~~%TO_~~%YO!Q~~%_P^~!Q![%Y~%gO!P~~%lPo~!_!`%O~%tRU~!c!}%o#R#S%o#T#o%o",
  tokenizers: [0],
  topRules: {"Program":[0,2]},
  specialized: [{term: 6, get: (value) => spec_Identifier[value] || -1}],
  tokenPrec: 0
})
