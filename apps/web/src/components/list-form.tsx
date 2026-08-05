import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import { Button } from "~/components/ui/button";
import { TextField, TextFieldInput, TextFieldLabel } from "~/components/ui/text-field";
import { type Anime, type ListVisibility, searchAnime } from "~/lib/list";

export type ListFormValue = {
    name: string;
    visibility: ListVisibility;
    animeIds: number[];
};

export function ListForm(props: {
    initialName?: string;
    initialVisibility?: ListVisibility;
    existingAnimeIds?: number[];
    shareUrl?: string;
    visibilityOnly?: boolean;
    submitLabel: string;
    onSubmit: (value: ListFormValue) => Promise<void>;
}) {
    const [name, setName] = createSignal(props.initialName ?? "");
    const [visibility, setVisibility] = createSignal<ListVisibility>(props.initialVisibility ?? "PRIVATE");
    const [selected, setSelected] = createSignal<Anime[]>([]);
    const [submitting, setSubmitting] = createSignal(false);
    const [error, setError] = createSignal<string>();

    const submit = async (event: SubmitEvent) => {
        event.preventDefault();
        const trimmedName = name().trim();
        if (!trimmedName) {
            setError("Enter a list name.");
            return;
        }

        setSubmitting(true);
        setError();
        try {
            await props.onSubmit({
                name: trimmedName,
                visibility: visibility(),
                animeIds: selected().map(({ id }) => id),
            });
        } catch (cause) {
            setError(cause instanceof Error ? cause.message : "Unable to save list.");
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <form class="mt-6 flex flex-col gap-5" onSubmit={submit}>
            <Show when={!props.visibilityOnly}>
                <TextField
                    value={name()}
                    onChange={setName}
                    required
                    validationState={error() && !name().trim() ? "invalid" : "valid"}
                >
                    <TextFieldLabel>List name</TextFieldLabel>
                    <TextFieldInput maxlength={100} placeholder="Weekend favorites" />
                </TextField>
            </Show>

            <label class="flex flex-col gap-1 text-sm font-medium">
                Visibility
                <select
                    class="h-10 rounded-md border border-input bg-background px-3 text-sm focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                    value={visibility()}
                    onChange={(event) => setVisibility(event.currentTarget.value as ListVisibility)}
                >
                    <option value="PRIVATE">Private</option>
                    <option value="UNLISTED">Unlisted</option>
                    <option value="PUBLIC">Public</option>
                </select>
                <span class="font-normal text-muted-foreground">
                    {visibility() === "PRIVATE"
                        ? "Only you can view this list."
                        : "Anyone with the link can view this list."}
                </span>
                <Show when={props.shareUrl && visibility() !== "PRIVATE"}>
                    <a class="mt-1 w-fit font-normal text-primary underline underline-offset-4" href={props.shareUrl}>
                        Open shareable list
                    </a>
                </Show>
            </label>

            <Show when={!props.visibilityOnly}>
                <AnimePicker
                    selected={selected()}
                    excludedIds={props.existingAnimeIds ?? []}
                    onSelect={(anime) => setSelected((current) => [...current, anime])}
                    onRemove={(animeId) => setSelected((current) => current.filter(({ id }) => id !== animeId))}
                />
            </Show>

            <Show when={error()}>
                {(message) => (
                    <p class="text-sm text-destructive" role="alert">
                        {message()}
                    </p>
                )}
            </Show>
            <Button type="submit" disabled={submitting()}>
                {submitting() ? "Saving..." : props.submitLabel}
            </Button>
        </form>
    );
}

function AnimePicker(props: {
    selected: Anime[];
    excludedIds: number[];
    onSelect: (anime: Anime) => void;
    onRemove: (animeId: number) => void;
}) {
    const [query, setQuery] = createSignal("");
    const [results, setResults] = createSignal<Anime[]>([]);
    const [searching, setSearching] = createSignal(false);
    const [searchError, setSearchError] = createSignal<string>();
    let searchVersion = 0;

    createEffect(() => {
        const version = ++searchVersion;
        const value = query().trim().slice(0, 30);
        if (value.length < 2) {
            setResults([]);
            setSearching(false);
            setSearchError();
            return;
        }

        setSearching(true);
        const timeout = setTimeout(async () => {
            try {
                const anime = await searchAnime(value);
                if (version === searchVersion) {
                    setResults(anime);
                    setSearchError();
                }
            } catch (cause) {
                if (version === searchVersion) {
                    setSearchError(cause instanceof Error ? cause.message : "Anime search failed.");
                }
            } finally {
                if (version === searchVersion) setSearching(false);
            }
        }, 250);
        onCleanup(() => clearTimeout(timeout));
    });

    const unavailableIds = () => new Set([...props.excludedIds, ...props.selected.map(({ id }) => id)]);

    return (
        <section class="flex flex-col gap-3" aria-labelledby="anime-picker-label">
            <TextField value={query()} onChange={(value) => setQuery(value.slice(0, 30))}>
                <TextFieldLabel id="anime-picker-label">Add anime</TextFieldLabel>
                <TextFieldInput type="search" placeholder="Search local anime" maxlength={30} />
            </TextField>
            <Show when={props.selected.length > 0}>
                <ul class="flex flex-wrap gap-2" aria-label="Selected anime">
                    <For each={props.selected}>
                        {(anime) => (
                            <li>
                                <Button
                                    type="button"
                                    size="sm"
                                    variant="secondary"
                                    onClick={() => props.onRemove(anime.id)}
                                >
                                    {anime.englishTitle ?? anime.romajiTitle}
                                    <span aria-hidden="true">×</span>
                                    <span class="sr-only">Remove</span>
                                </Button>
                            </li>
                        )}
                    </For>
                </ul>
            </Show>
            <div class="max-h-64 overflow-y-auto rounded-md border border-border" aria-live="polite">
                <Show when={!searching()} fallback={<p class="p-3 text-sm text-muted-foreground">Searching...</p>}>
                    <Show when={!searchError()} fallback={<p class="p-3 text-sm text-destructive">{searchError()}</p>}>
                        <For
                            each={results().filter(({ id }) => !unavailableIds().has(id))}
                            fallback={
                                query().trim().length >= 2 ? (
                                    <p class="p-3 text-sm text-muted-foreground">No matching anime found.</p>
                                ) : (
                                    <p class="p-3 text-sm text-muted-foreground">
                                        Type at least two characters to search.
                                    </p>
                                )
                            }
                        >
                            {(anime) => (
                                <button
                                    type="button"
                                    class="flex w-full items-center gap-3 border-b border-border p-2 text-left last:border-0 hover:bg-accent focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
                                    onClick={() => props.onSelect(anime)}
                                >
                                    <Show when={anime.picture}>
                                        {(picture) => (
                                            <img src={picture()} alt="" class="h-12 w-9 rounded object-cover" />
                                        )}
                                    </Show>
                                    <span class="min-w-0">
                                        <span class="block truncate text-sm font-medium">
                                            {anime.englishTitle ?? anime.romajiTitle}
                                        </span>
                                        <Show when={anime.englishTitle}>
                                            <span class="block truncate text-xs text-muted-foreground">
                                                {anime.romajiTitle}
                                            </span>
                                        </Show>
                                    </span>
                                </button>
                            )}
                        </For>
                    </Show>
                </Show>
            </div>
        </section>
    );
}
