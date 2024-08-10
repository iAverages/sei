import { createMutation, useQueryClient } from "@tanstack/solid-query";

export const createUpdateListOrder = () => {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationKey: ["anime", "list", "update"],
    mutationFn: async (ids: number[]) => {
      const res = await fetch(
        `${import.meta.env.PUBLIC_API_URL ?? ""}/api/v1/user/list`,
        {
          method: "POST",
          credentials: "include",
          body: JSON.stringify({ ids }),
          headers: {
            "Content-Type": "application/json",
          },
        },
      );

      if (!res.ok) {
        throw res;
      }

      return res;
    },
    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: ["anime", "list"],
      });
    },
  }));
};
