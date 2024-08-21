import { parser } from "../src/parser.js";

const code = "true";

const tree = parser.parse(code);
const treeOutp = JSON.stringify(tree, null, 4);
console.log(treeOutp);
