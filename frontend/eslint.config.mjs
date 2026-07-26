import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import angular from "angular-eslint";

const banDeepSelectorsRule = {
    meta: {
        type: "problem",
        docs: {
            description: "Ban ::ng-deep, ::v-deep, and /deep/ in TypeScript files",
        },
        schema: [],
    },
    create(context) {
        const forbidden = ["::ng-deep", "::v-deep", "/deep/"];
        return {
            Literal(node) {
                if (typeof node.value === "string") {
                    for (const term of forbidden) {
                        if (node.value.includes(term)) {
                            context.report({
                                node,
                                message: `Usage of ${term} is strictly forbidden.`,
                            });
                        }
                    }
                }
            },
            TemplateLiteral(node) {
                for (const quasi of node.quasis) {
                    for (const term of forbidden) {
                        if (quasi.value.raw.includes(term)) {
                            context.report({
                                node,
                                message: `Usage of ${term} is strictly forbidden.`,
                            });
                        }
                    }
                }
            }
        };
    }
};

const sortClassMembersRule = {
    meta: {
        type: "suggestion",
        docs: {
            description: "Enforce sorting of class members: private first, then protected, then public. First inject() properties, then constructor.",
        },
        fixable: "code",
        schema: [],
    },
    create(context) {
        const sourceCode = context.sourceCode || context.getSourceCode();

        function isInputProperty(node) {
            if (!node || node.type !== "PropertyDefinition") return false;
            if (node.value && node.value.type === "CallExpression") {
                const callee = node.value.callee;
                if (callee.type === "Identifier" && callee.name === "input") return true;
                if (callee.type === "MemberExpression" && callee.object && callee.object.type === "Identifier" && callee.object.name === "input") return true;
            }
            if (node.decorators) {
                for (const dec of node.decorators) {
                    if (dec.expression && (dec.expression.name === "Input" || (dec.expression.callee && dec.expression.callee.name === "Input"))) {
                        return true;
                    }
                }
            }
            return false;
        }

        function getMemberRank(node) {
            const isStatic = node.static === true;
            
            let access = 'public';
            if (node.accessibility === 'private') {
                access = 'private';
            } else if (node.accessibility === 'protected') {
                access = 'protected';
            } else if (node.accessibility === 'public') {
                access = 'public';
            }

            const isConstructor = (node.type === 'MethodDefinition' && node.kind === 'constructor') ||
                                  (node.type === 'TSDeclareFunction' && node.key && node.key.name === 'constructor');
            const isMethod = node.type === 'MethodDefinition' || node.type === 'TSDeclareFunction';
            const isProperty = node.type === 'PropertyDefinition' || node.type === 'ClassProperty';

            let isInject = false;
            if (isProperty && node.value && node.value.type === 'CallExpression') {
                const callee = node.value.callee;
                if (callee && callee.type === 'Identifier' && callee.name === 'inject') {
                    isInject = true;
                }
            }

            if (isInject) {
                let injectBase = 1;
                if (access === 'private') return injectBase + 0;   // 1
                if (access === 'protected') return injectBase + 1; // 2
                return injectBase + 2;                             // 3
            }

            if (isInputProperty(node)) {
                return 5;
            }

            if (isConstructor) {
                return 15;
            }

            if (isStatic) {
                let staticBase = 0;
                if (access === 'private') staticBase = 1;
                else if (access === 'protected') staticBase = 2;
                else if (access === 'public') staticBase = 3;

                if (isProperty) return staticBase;
                if (isMethod) return staticBase + 3;
            } else {
                let propBase = 10;
                let methodBase = 20;

                let offset = 2; // public
                if (access === 'private') offset = 0;
                else if (access === 'protected') offset = 1;

                if (isProperty) {
                    return propBase + offset;
                }
                if (isMethod) {
                    return methodBase + offset;
                }
            }
            return 50;
        }

        function getMemberName(node) {
            if (node.key) {
                if (node.key.type === 'Identifier') {
                    return node.key.name;
                }
                if (node.key.type === 'Literal') {
                    return String(node.key.value);
                }
            }
            if (node.type === 'MethodDefinition' && node.kind === 'constructor') {
                return 'constructor';
            }
            return 'unknown';
        }

        return {
            ClassBody(node) {
                const members = node.body.filter(m => m.type !== 'EmptyStatement');
                if (members.length <= 1) return;

                // Check if sorted
                let lastRank = -1;
                let outOfOrderNode = null;

                for (const member of members) {
                    const rank = getMemberRank(member);
                    if (rank < lastRank) {
                        outOfOrderNode = member;
                        break;
                    }
                    lastRank = rank;
                }

                if (outOfOrderNode) {
                    context.report({
                        node: outOfOrderNode,
                        message: `Class member '${getMemberName(outOfOrderNode)}' is out of order. Enforce private first, then protected, then public. First inject() properties, then constructor, then methods.`,
                        fix(fixer) {
                            const memberTexts = [];
                            let lastEnd = node.range[0] + 1;

                            for (let i = 0; i < members.length; i++) {
                                const current = members[i];
                                const start = lastEnd;
                                const end = current.range[1];
                                memberTexts.push({
                                    node: current,
                                    text: sourceCode.text.slice(start, end),
                                    rank: getMemberRank(current),
                                    index: i
                                });
                                lastEnd = end;
                            }

                            const remainingText = sourceCode.text.slice(lastEnd, node.range[1] - 1);

                            const sortedMemberTexts = [...memberTexts].sort((a, b) => {
                                if (a.rank !== b.rank) {
                                    return a.rank - b.rank;
                                }
                                return a.index - b.index;
                            });

                            let newBodyText = "";
                            for (const m of sortedMemberTexts) {
                                newBodyText += m.text;
                            }
                            newBodyText += remainingText;

                            return fixer.replaceTextRange([node.range[0] + 1, node.range[1] - 1], newBodyText);
                        }
                    });
                }
            }
        };
    }
};

const inputsBeforeUsageRule = {
    meta: {
        type: "problem",
        docs: {
            description: "Enforce signal inputs/Inputs to be declared above any class members that use them.",
        },
        schema: [],
    },
    create(context) {
        function isInputProperty(node) {
            if (!node || node.type !== "PropertyDefinition") return false;
            if (node.value && node.value.type === "CallExpression") {
                const callee = node.value.callee;
                if (callee.type === "Identifier" && callee.name === "input") return true;
                if (callee.type === "MemberExpression" && callee.object && callee.object.type === "Identifier" && callee.object.name === "input") return true;
            }
            if (node.decorators) {
                for (const dec of node.decorators) {
                    if (dec.expression && (dec.expression.name === "Input" || (dec.expression.callee && dec.expression.callee.name === "Input"))) {
                        return true;
                    }
                }
            }
            return false;
        }

        function getMemberName(node) {
            if (node.key) {
                if (node.key.type === "Identifier") return node.key.name;
                if (node.key.type === "Literal") return String(node.key.value);
            }
            return "member";
        }

        function findThisReferences(node, targetName) {
            const references = [];
            function traverse(n) {
                if (!n || typeof n !== "object") return;
                if (Array.isArray(n)) {
                    for (const child of n) traverse(child);
                    return;
                }
                if (
                    n.type === "MemberExpression" &&
                    n.object &&
                    n.object.type === "ThisExpression" &&
                    n.property &&
                    n.property.type === "Identifier" &&
                    n.property.name === targetName
                ) {
                    references.push(n);
                }
                for (const key of Object.keys(n)) {
                    if (key === "parent") continue;
                    traverse(n[key]);
                }
            }
            traverse(node);
            return references;
        }

        return {
            ClassBody(node) {
                const members = node.body.filter(m => m.type !== "EmptyStatement");
                const inputsMap = new Map();

                for (let i = 0; i < members.length; i++) {
                    const member = members[i];
                    if (isInputProperty(member)) {
                        const inputName = getMemberName(member);
                        inputsMap.set(inputName, { node: member, index: i });
                    }
                }

                if (inputsMap.size === 0) return;

                for (let i = 0; i < members.length; i++) {
                    const member = members[i];
                    const memberName = getMemberName(member);

                    for (const [inputName, inputInfo] of inputsMap.entries()) {
                        if (member === inputInfo.node) continue;
                        if (i < inputInfo.index) {
                            const refs = findThisReferences(member, inputName);
                            if (refs.length > 0) {
                                context.report({
                                    node: refs[0],
                                    message: `Input property '${inputName}' must be declared above '${memberName}' (or any members that use it).`,
                                });
                            }
                        }
                    }
                }
            }
        };
    }
};

const localPlugin = {
    rules: {
        "ban-deep-selectors": banDeepSelectorsRule,
        "sort-class-members": sortClassMembersRule,
        "inputs-before-usage": inputsBeforeUsageRule
    }
};

const scssFilesText = new Map();
const scssProcessor = {
    preprocess(text, filename) {
        scssFilesText.set(filename, text);
        return [{ text: "/* dummy */", filename: "dummy.js" }];
    },
    postprocess(messages, filename) {
        const text = scssFilesText.get(filename);
        scssFilesText.delete(filename);

        const errors = [];
        if (text) {
            const forbidden = [
                { pattern: "::ng-deep", message: "Usage of ::ng-deep is strictly forbidden." },
                { pattern: "::v-deep", message: "Usage of ::v-deep is strictly forbidden." },
                { pattern: "/deep/", message: "Usage of /deep/ is strictly forbidden." }
            ];
            const lines = text.split("\n");
            for (let i = 0; i < lines.length; i++) {
                for (const item of forbidden) {
                    const index = lines[i].indexOf(item.pattern);
                    if (index !== -1) {
                        errors.push({
                            ruleId: "local/ban-deep-selectors",
                            severity: 2,
                            message: item.message,
                            line: i + 1,
                            column: index + 1,
                        });
                    }
                }
            }
        }
        return errors;
    }
};

export default tseslint.config(
    {
        files: ["**/*.ts"],
        plugins: {
            local: localPlugin
        },
        extends: [
            eslint.configs.recommended,
            ...tseslint.configs.recommended,
            ...tseslint.configs.stylistic,
            ...angular.configs.tsRecommended,
        ],
        processor: angular.processInlineTemplates,
        rules: {
            "@angular-eslint/directive-selector": [
                "error",
                {
                    type: "attribute",
                    prefix: "app",
                    style: "camelCase",
                },
            ],
            "@angular-eslint/component-selector": [
                "error",
                {
                    type: "element",
                    prefix: "app",
                    style: "kebab-case",
                },
            ],
            "@typescript-eslint/no-unused-vars": ["warn", {
                "argsIgnorePattern": "^_",
                "varsIgnorePattern": "^_",
                "caughtErrorsIgnorePattern": "^_"
            }],
            "@typescript-eslint/no-empty-function": "off",
            "@typescript-eslint/consistent-type-definitions": "off",
            "@typescript-eslint/no-explicit-any": "error",
            "local/ban-deep-selectors": "error",
            "local/sort-class-members": "error",
            "local/inputs-before-usage": "error",
            "no-restricted-syntax": [
                "error",
                {
                    "selector": "MemberExpression[object.name='ViewEncapsulation'][property.name='None']",
                    "message": "Usage of ViewEncapsulation.None is strictly forbidden."
                }
            ],
        },
    },
    {
        files: ["**/*.html"],
        extends: [
            ...angular.configs.templateRecommended,
            ...angular.configs.templateAccessibility,
        ],
        rules: {
            "@angular-eslint/template/prefer-self-closing-tags": "error",
            "@angular-eslint/template/label-has-associated-control": ["error", {
                "controlComponents": [
                    "input",
                    "textarea",
                    "select",
                    "meter",
                    "progress",
                    "output",
                    "app-custom-input",
                    "my-awesome-select"
                ]
            }]
        },
    },
    {
        files: ["**/*.scss", "**/*.css"],
        processor: scssProcessor,
    }
);