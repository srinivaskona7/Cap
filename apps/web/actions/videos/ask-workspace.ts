"use server";

import { getCurrentUser } from "@cap/database/auth/session";
import type { User } from "@cap/web-domain";
import { searchWorkspaceMcpCaps } from "@/lib/mcp-data";

export async function askWorkspaceVideos(query: string) {
	const user = await getCurrentUser();
	if (!user) {
		throw new Error("Unauthorized");
	}
	return searchWorkspaceMcpCaps(user.id as User.UserId, {
		query,
		limit: 8,
	});
}
