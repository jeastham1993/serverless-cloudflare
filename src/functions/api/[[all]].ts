interface Env {
	BACKEND: Fetcher;
}

export const onRequest: PagesFunction<Env> = async (context) => {
	const origin_data = await context.env.BACKEND.fetch(context.request);

	return origin_data;
}