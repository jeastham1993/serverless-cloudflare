import { Fetcher, PagesFunction } from "@cloudflare/workers-types/experimental";

interface Env {
	AUTH: Fetcher;
}

export const onRequest: PagesFunction<Env> = async (context) => {
	const origin_data = await context.env.AUTH.fetch(context.request);

	return origin_data;
}