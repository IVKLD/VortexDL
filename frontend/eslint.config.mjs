import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import angular from "angular-eslint";

const banNgDeepRule = {
    meta: {
        type: "problem",
        docs: {
            description: "Ban ::ng-deep in TypeScript files",
        },
        schema: [],
    },
    create(context) {
        return {
            Literal(node) {
                if (typeof node.value === "string" && node.value.includes("::ng-deep")) {
                    context.report({
                        node,
                        message: "Usage of ::ng-deep is strictly forbidden.",
                    });
                }
            },
            TemplateLiteral(node) {
                const hasNgDeep = node.quasis.some(quasi => quasi.value.raw.includes("::ng-deep"));
                if (hasNgDeep) {
                    context.report({
                        node,
                        message: "Usage of ::ng-deep is strictly forbidden.",
                    });
                }
            }
        };
    }
};

const localPlugin = {
    rules: {
        "ban-ng-deep": banNgDeepRule
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
        if (text && text.includes("::ng-deep")) {
            const lines = text.split("\n");
            for (let i = 0; i < lines.length; i++) {
                const index = lines[i].indexOf("::ng-deep");
                if (index !== -1) {
                    errors.push({
                        ruleId: "local/ban-ng-deep",
                        severity: 2,
                        message: "Usage of ::ng-deep is strictly forbidden.",
                        line: i + 1,
                        column: index + 1,
                    });
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
            "local/ban-ng-deep": "error",
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