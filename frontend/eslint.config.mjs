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

        function getMemberRank(node) {
            const isStatic = node.static === true;
            
            // Check accessibility
            let access = 'public';
            if (node.accessibility === 'private') {
                access = 'private';
            } else if (node.accessibility === 'protected') {
                access = 'protected';
            } else if (node.accessibility === 'public') {
                access = 'public';
            }

            // Check kind
            const isConstructor = (node.type === 'MethodDefinition' && node.kind === 'constructor') ||
                                  (node.type === 'TSDeclareFunction' && node.key && node.key.name === 'constructor');
            const isMethod = node.type === 'MethodDefinition' || node.type === 'TSDeclareFunction';
            const isProperty = node.type === 'PropertyDefinition' || node.type === 'ClassProperty';

            // Check if it's an inject property
            let isInject = false;
            if (isProperty && node.value && node.value.type === 'CallExpression') {
                const callee = node.value.callee;
                if (callee && callee.type === 'Identifier' && callee.name === 'inject') {
                    isInject = true;
                }
            }

            if (isConstructor) {
                return 30; // constructor is public/constructor, rank 30
            }

            if (isStatic) {
                let staticBase = 0;
                if (access === 'private') staticBase = 1;
                else if (access === 'protected') staticBase = 2;
                else if (access === 'public') staticBase = 3;

                if (isProperty) return staticBase;      // 1, 2, 3
                if (isMethod) return staticBase + 3;    // 4, 5, 6
            } else {
                if (isProperty) {
                    if (isInject) {
                        let injectBase = 10;
                        if (access === 'private') return injectBase + 0;   // 10
                        if (access === 'protected') return injectBase + 1; // 11
                        return injectBase + 2;                             // 12
                    } else {
                        let otherBase = 20;
                        if (access === 'private') return otherBase + 0;   // 20
                        if (access === 'protected') return otherBase + 1; // 21
                        return otherBase + 2;                             // 22
                    }
                }
                if (isMethod) {
                    let methodBase = 40;
                    if (access === 'private') return methodBase + 0;   // 40
                    if (access === 'protected') return methodBase + 1; // 41
                    return methodBase + 2;                             // 42
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

const localPlugin = {
    rules: {
        "ban-deep-selectors": banDeepSelectorsRule,
        "sort-class-members": sortClassMembersRule
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
            "@typescript-eslint/consistent-type-definitions": "off",
            "@typescript-eslint/no-explicit-any": "error",
            "local/ban-deep-selectors": "error",
            "local/sort-class-members": "error",
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