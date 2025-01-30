// This file was generated by lezer-generator. You probably shouldn't edit it.
import {LRParser} from "@lezer/lr"
const spec_Identifier = {__proto__:null,if:98, IF:98, new:100, NEW:100, Void:102, Int:102, String:102, Boolean:102, Array:102, Collection:102, Stack:102, Queue:102, true:110, false:110, not:112, NOT:112, then:114, THEN:114, else:116, ELSE:116, end:118, END:118, function:120, FUNCTION:120, output:126, OUTPUT:126, return:128, RETURN:128, loop:130, LOOP:130, for:132, FOR:132, from:134, FROM:134, to:136, TO:136, while:138, WHILE:138}
export const parser = LRParser.deserialize({
  version: 14,
  states: "-OQYQPOOOOQO'#Ca'#CaO#bQPO'#CdOOQO'#Ci'#CiOOQO'#Cj'#CjO#iQPO'#ChO#nQPO'#DWOOQO'#Cl'#ClOOQO'#Cp'#CpOOQO'#Cc'#CcO$xQPO'#C`OOQO'#Cv'#CvO%gQPO'#CuOOQO'#Cz'#CzO%lQPO'#CyOOQO'#C|'#C|O%lQPO'#C{OOQO'#DO'#DOO&WQPO'#DSOOQO'#C_'#C_OOQO'#DX'#DXQYQPOOO&`QPO,59QO%lQPO,59pO&gQPO,59SO&lQPO,59VO&qQPO'#CdO&{QPO'#DYO$xQPO,58zOOQO'#Cq'#CqO'mQPO,58zO'wQPO,59aO'|QPO,59eO)WQPO,59gOOQO'#DP'#DPO*bQPO,59iOOQO'#DT'#DTO%lQPO,59nOOQO-E7V-E7VOOQO1G.l1G.lO*gQPO1G.lO*nQPO1G/[O#iQPO1G.nO+xQPO1G.qOOQO1G.q1G.qOOQO-E7W-E7WO'mQPO1G.fO-]QPO'#CrO-gQPO1G.fO-oQPO'#CwO'pQPO1G.{O-wQPO1G/TO-|QPO1G/YOOQO7+$W7+$WO.WQPO7+$YO-gQPO7+$QOOQO'#Cs'#CsOOQO'#Ct'#CtOOQO7+$Q7+$QO'pQPO7+$QO.]QPO'#CxOOQO,59c,59cO.bQPO,59cO-jQPO7+$gOOQO'#DQ'#DQO%lQPO7+$oO-jQPO7+$tO.jQPO<<GtOOQO<<Gl<<GlO'pQPO<<GlO-jQPO<<GlO#iQPO,59dO.oQPO'#DZO.tQPO1G.}OOQO1G.}1G.}OOQO<<HR<<HRO.|QPO<<HZOOQO<<H`<<H`O/UQPOAN=`O-jQPOAN=WOOQOAN=WAN=WOOQO1G/O1G/OOOQO,59u,59uOOQO-E7X-E7XOOQO7+$i7+$iOOQO'#DR'#DRO%lQPOAN=uOOQOG22zG22zOOQOG22rG22rO-|QPOG23aO-jQPOLD({OOQO!$'Lg!$'Lg",
  stateData: "/Z~O!QOSPOS~OUQOaXObXOcXO!RPO!SRO!TSO!XVO!YWO!^ZO!a]O!b_O!caO~OZfOUWXaWXbWXcWX!OWX!RWX!SWX!TWX!WWX!XWX!YWX!^WX!aWX!bWX!cWXXWX![WX!]WX~OygO~P!TO!TSO~O!WiOUzXazXbzXczX!OzX!RzX!SzX!TzX!XzX!YzX!^zX!azX!bzX!czXXzX![zX!]zX~OUjOaXObXOcXO!SRO!TSO!XVO!YWO!ZmO~OUoO~OUjOaXObXOcXO!SRO!TSO!XVO!YWO~O!drO!gtO~OXwO~PYO!UzO~OU{O~O!ZWX!fWX~P!TO!WiOU|Xa|Xb|Xc|X!S|X!T|X!X|X!Y|X!Z|X~O![fP!]fP~PYOZ!RO~O!WiOUmaamabmacma!Oma!Rma!Sma!Tma!Xma!Yma!^ma!ama!bma!cmaXma![ma!]ma~O!WiOUoaaoaboacoa!Ooa!Roa!Soa!Toa!Xoa!Yoa!^oa!aoa!boa!coaXoa![oa!]oa~OU!TO~OX!VO~PYO!WiOUxiaxibxicxi!Oxi!Rxi!Sxi!Txi!Xxi!Yxi!^xi!axi!bxi!cxiXxi![xi!]xi~OZfOU_ia_ib_ic_i!O_i!R_i!S_i!T_i!W_i!X_i!Y_i!^_i!a_i!b_i!c_i!Z_iX_i![_i!]_i!f_i~O![fX!]fX~PYO![!YO!]!ZO~OU!^OX!_O~O!e!bO~O!WiO!]fP~PYO!V!eO~O!_!iO~OX!lO!`!jO~OZ!pO~OU!^O~OX!vO!`!jO~O!WiO!f!wO~OX!yO~O",
  goto: ")d!OPPP!P!_!mP!{#qP$XP#q$r%Y#q#qPPP#q%y&P&f&l!_'R'a'd!_'j!_'x!_(W(f(i(l!_(o!_P!_(r)W)^idOefnx!O!P!S!U!]!g!{icOefnx!O!P!S!U!]!g!{iYOefnx!O!P!S!U!]!g!{hUOefnx!O!P!S!U!]!g!{SkYlQp^Qq`QygQ!UuQ!n!cR!{!xyXOY^`efglnux!O!P!S!U!]!c!g!x!{xXOY^`efglnux!O!P!S!U!]!c!g!x!{R|iyTOY^`efglnux!O!P!S!U!]!c!g!x!{xXOY^`efglnux!O!P!S!U!]!c!g!x!{QhTQ!WzR!s!iQnYR!OlQ!QnQ!X!OQ!a!SQ!d!UQ!h!]Q!q!gR!|!{Q!]!QR!g!XQ![!QQ!f!XQ!m!aQ!o!dQ!r!hQ!z!qR!}!|i[Oefnx!O!P!S!U!]!g!{R!SoQ!`!RR!t!ji^Oefnx!O!P!S!U!]!g!{i`Oefnx!O!P!S!U!]!g!{ibOefnx!O!P!S!U!]!g!{RsbR!c!TR!x!nRubQeOUvex!PQxf_!Pn!O!S!U!]!g!{QlYR}lQ!k!`R!u!k",
  nodeNames: "⚠ LineComment Program Atom IfStatement IfKeyword Identifier Expression ReferenceEpxression ) CallExpression ( ObjectInstantiationExpression NewKeyword TypeAnnotation MemberAccessExpression Boolean String Number MiscOperator NotKeyword ThenKeyword Block ElseKeyword EndKeyword FunctionDeclaration FunctionKeyword ParameterList Parameter OutputStatement OutputKeyword ReturnStatement ReturnKeyword ForStatement LoopKeyword ForKeyword FromKeyword ToKeyword WhileStatement WhileKeyword VariableAssignment AssignmentOperator ExpressionStatement",
  maxTerm: 69,
  nodeProps: [
    ["openedBy", 9,"("],
    ["closedBy", 11,")"]
  ],
  skippedNodes: [0,1],
  repeatNodeCount: 3,
  tokenData: "&g~RfXY!gYZ!g]^!gpq!grs!xst$fxy$}yz%Sz{%X{|%X|}%^}!O%X!O!P%c!P!Q%X!Q![%h![!]%p!^!_%u!_!`%z!`!a&S!c!}&X#R#S&X#T#o&X~!lS!Q~XY!gYZ!g]^!gpq!g~!{VOr!xrs#bs#O!x#O#P#g#P;'S!x;'S;=`$`<%lO!x~#gOa~~#jRO;'S!x;'S;=`#s;=`O!x~#vWOr!xrs#bs#O!x#O#P#g#P;'S!x;'S;=`$`;=`<%l!x<%lO!x~$cP;=`<%l!x~$kSP~OY$fZ;'S$f;'S;=`$w<%lO$f~$zP;=`<%l$f~%SOZ~~%XOX~~%^Oc~~%cO!`~~%hO!W~~%mPb~!Q![%h~%uO!_~~%zO!U~~&PPy~!_!`%X~&XO!V~~&^RU~!c!}&X#R#S&X#T#o&X",
  tokenizers: [0],
  topRules: {"Program":[0,2]},
  specialized: [{term: 6, get: (value) => spec_Identifier[value] || -1}],
  tokenPrec: 0
})
