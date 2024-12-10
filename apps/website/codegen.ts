import { CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
	schema: "./schema.graphql",
	overwrite: true,
	documents: ["./src/**/*.svelte", "./src/**/*.graphql", "./src/**/*.ts"],
	generates: {
		"./src/gql/": {
			preset: "client",
			config: {
				useTypeImports: true,
				strictScalars: true,
				scalars: {
					DateTime: "Date",
					Id: "string",
					ProductId: "string",
					InvoiceId: "string",
					JSONObject: "object",
				},
			},
		},
	},
};

export default config;
