# Terraform will panic if it encounters `PGSERVICEFILE`.

unexport PGSERVICEFILE

os := "darwin-aarch64"
plan_dir := justfile_directory() / ".terraform-plans"

_default:
    @just fmt
    @just lint
    @just test

# ------------------------------------------------------------------------------
# Docs

# Create a new decision record
[group('docs')]
decide +title:
    #!/usr/bin/env bash
    timestamp=$(date +%Y%m%d%H%M%S)
    normalized=$(echo "{{ title }}" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g' | sed 's/^-\|-$//g')
    author="$(git config user.name) <$(git config user.email)>"
    filename="decisions/${timestamp}-${normalized}.org"
    cat > "$filename" <<EOF
    #+title:  {{ title }}
    #+author: ${author}
    #+date:   $(date +%Y-%m-%d)
    EOF
    echo "🎯 {{ BOLD }}Created \"$filename\"{{ NORMAL }}."

# Create a new prompt
[group('docs')]
prompt +title:
    #!/usr/bin/env zsh
    timestamp=$(date +%Y%m%d%H%M%S)
    normalized=$(echo "{{ title }}" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g' | sed 's/^-\|-$//g')
    filename=".claude/prompts/${timestamp}-${normalized}.org"
    mkdir -p .claude/prompts
    cat > "$filename" <<EOF
    #+title:    {{ title }}
    #+date:     $(date +%Y-%m-%d)
    #+status:   todo
    #+property: header-args :dir ../..
    EOF
    echo "💭 {{ BOLD }}Created \"$filename\"{{ NORMAL }}."

# Execute a prompt in a new Claude Code session
[group('docs')]
execute *args:
    #!/usr/bin/env zsh
    setopt NULL_GLOB
    set -e

    # Check for --all flag and extract query
    show_all=false
    query_args="{{ args }}"

    if [[ "$query_args" == *"--all"* ]]; then
        show_all=true
        query_args="${query_args//--all/}"
        # Clean up extra whitespace
        query_args="${query_args## }"
        query_args="${query_args%% }"
    fi

    get_status_indicator() {
        local prompt_status="$1"
        case "$prompt_status" in
            done) echo "[✓]" ;;
            doing) echo "[~]" ;;
            todo) echo "[ ]" ;;
            *) echo "[ ]" ;;
        esac
    }

    prompt_list=""
    for file in .claude/prompts/*.org(N); do
        [[ ! -f "$file" ]] && continue

        basename=$(basename "$file" .org)
        title=$(grep -m1 '^#+title:' "$file" | sed 's/#+title: *//' | xargs)
        prompt_status=$(grep -m1 '^#+status:' "$file" | sed 's/#+status: *//' | xargs)
        [[ -z "$prompt_status" ]] && prompt_status="todo"

        # Skip completed prompts unless --all is passed
        [[ "$prompt_status" == "done" && "$show_all" == "false" ]] && continue

        # Extract date from filename (YYYYMMDDHHMMSS format)
        timestamp="${basename:0:14}"
        year="${timestamp:0:4}"
        month="${timestamp:4:2}"
        day="${timestamp:6:2}"
        hour="${timestamp:8:2}"
        min="${timestamp:10:2}"
        sec="${timestamp:12:2}"
        date_formatted="${year}/${month}/${day} ${hour}:${min}:${sec}"

        indicator=$(get_status_indicator "$prompt_status")
        prompt_list+="${indicator}  ${date_formatted}  ${title}  ${basename}\n"
    done

    if [[ -z "$prompt_list" ]]; then
        echo >&2 "No prompts found in .claude/prompts/"
        exit 1
    fi

    # Build fzf arguments array
    fzf_args=(
        --height=40%
        --reverse
        --with-nth=1,2,3
        --delimiter='  '
        --prompt="Select prompt: "
        --preview='cat .claude/prompts/{-1}.org'
        --preview-window=right:60%:wrap
    )

    # Add query if provided
    if [[ -n "$query_args" ]]; then
        fzf_args+=(--query="$query_args")
    fi

    selected=$(printf "%b" "$prompt_list" | fzf "${fzf_args[@]}")

    prompt_slug=$(echo "$selected" | awk -F'  ' '{print $NF}')
    prompt_file=".claude/prompts/${prompt_slug}.org"
    title=$(grep -m1 '^#+title:' "$prompt_file" | sed 's/#+title: *//')

    echo >&2 "{{ BOLD }}Executing {{ YELLOW }}\"${title}\"...{{ NORMAL }}"

    message="
    # $title

    Please read the prompt document at '$prompt_file', review the plan, and
    formulate a suitable plan to execute.

    ## Process

    1. Read the prompt - Understand goals and success criteria
    2. Consider improvements - Better approaches? Edge cases?
    3. Derisk assumptions - What could go wrong?
    4. Formulate a plan - Concrete steps with checkpoints
    5. Present the plan - Show your approach before executing
    6. Execute - Implement changes systematically
    7. Verify - Run tests to confirm success

    ## Important

    - Follow all guidelines in CLAUDE.md
    - Run tests after changes
    - Keep commits focused
    - Update the prompt doc with important changes and on-going status

    Ready to begin?"

    exec claude "$message"

# ------------------------------------------------------------------------------
# Development

# Create self-signed SSL certificates via mkcert
[group('dev')]
mkcert:
    #!/usr/bin/env zsh
    set -e

    mkcert -install
    mkdir -p {{ justfile_directory() }}/certs
    cd {{ justfile_directory() }}/certs

    # We append .test to all domains during development. The following domains
    # need to be supported:
    #
    # -- usebits.app
    # api.usebits.app
    # edit.usebits.app
    # www.usebits.app
    #
    # -- bits.page
    # bits.page
    # jcf.bits.page
    domains=(
        usebits.app.test
        app.test

        bits.page.test
        page.test

        test
    )

    for domain in $domains; do
        if [[ ! -f "_wildcard.${domain}.pem" ]]; then
            mkcert "*.${domain}"
        fi
    done

    echo >&2 "🔒 Wildcard certificates ready!"

# Generate cluster keystores
[group('dev')]
cluster-certs:
    #!/usr/bin/env zsh
    set -e
    mkdir -p certs

    if [[ ! -f certs/cluster-keystore.p12 ]]; then
        keytool -genkeypair \
            -alias bits-cluster \
            -keyalg EC \
            -groupname secp256r1 \
            -validity 3650 \
            -storetype PKCS12 \
            -keystore certs/cluster-keystore.p12 \
            -storepass "$CLUSTER_KEYSTORE_PASSWORD" \
            -dname "CN=bits-cluster,O=Invetica" \
            -ext "SAN=IP:127.0.0.1"

        keytool -genkeypair \
            -alias rogue \
            -keyalg EC \
            -groupname secp256r1 \
            -validity 1 \
            -storetype PKCS12 \
            -keystore certs/rogue-keystore.p12 \
            -storepass password \
            -dname "CN=rogue,O=Evil"
    fi

    echo >&2 "🔒 Cluster certificates ready!"

# Setup a local development environment
[group('dev')]
setup:
    @just mkcert
    @just cluster-certs
    devenv shell true
    pnpm install
    @echo -e "\n✅ {{ BOLD }}Setup complete!{{ NORMAL }}"

[group('dev')]
nrepl *args:
    #!/usr/bin/env zsh
    set -e

    echo >&2 \
        "{{ BLUE }}{{ BOLD }}==>{{ NORMAL }} {{ BOLD }}Starting nREPL on localhost:9999...{{ NORMAL }}"

    exec clojure \
        -M:dev:test:logging:nrepl:{{ os }} \
        --report stderr \
        --main bits.nrepl \
        --host localhost \
        --port 9999 \
        {{ args }}

# Watch source code for Tailwind classes
[group('dev')]
tailwind:
    mkdir -p resources/public
    pnpm --filter @bits/tailwind \
        exec tailwindcss \
            --watch \
            --input ../../resources/tailwind.css \
            --output ../../resources/public/app.css

# Run the marketing site
[group('dev')]
market:
    pnpm --filter @bits/www dev

# Regenerate Tailwind CSS from template
[group('dev')]
css:
    clojure -M:dev -m bits.dev.assets

# Format project files
[group('dev')]
fmt:
    treefmt

# Run lints
[group('dev')]
lint:
    clj-kondo --lint dev src test

# Run tests
[group('dev')]
test *args:
    clojure -M:dev:test:runner:{{ os }} {{ args }}

# Build an AOT-compiled uberjar
[group('build')]
build:
    clojure -T:build uber

# Build Datomic Pro output
[group('build')]
build-datomic:
    devenv build outputs.datomic-pro

# Build the Docker image
[group('build')]
docker-build tag="bits:latest":
    docker build -t {{ tag }} .

# Run the Docker image against the local devenv database
[group('build')]
docker-run tag="bits:latest" *args:
    docker run --rm -p 3000:3000 \
        -e DATABASE_URL=jdbc:postgresql://host.docker.internal:5432/bits_dev?user=bits\&password=please \
        -e CSRF_SECRET=default-csrf-secret-change-in-prod \
        --add-host=host.docker.internal:host-gateway \
        {{ args }} {{ tag }}

# ------------------------------------------------------------------------------
# PostgreSQL

# Start an interactive psql session connected to the local development database
[group('postgres')]
psql *args:
    PGPASSWD=please psql \
        --host=localhost \
        --port=5432 \
        --username=bits \
        --dbname=bits_dev \
        {{ args }}

# Create a new migration
[group('postgres')]
migration name:
    #!/usr/bin/env zsh
    set -e
    timestamp="$(date +%Y%m%d%H%M%S)"
    normalized=$(echo "{{ name }}" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g' | sed 's/^-\|-$//g')
    dir="resources/migrations"
    mkdir -p "$dir"
    for direction in up down; do
        path="$dir/${timestamp}-${normalized}.${direction}.sql"
        touch "$path" && echo "$path"
    done

# ------------------------------------------------------------------------------
# Infrastructure

[group('iac')]
_terraform dir *args:
    op run -- terraform -chdir={{ justfile_directory() }}/iac/{{ dir }} {{ args }}

# Initialize one or all Terraform projects
[group('iac')]
init dir *args:
    @just _terraform {{ dir }} init {{ args }}

# Plan one or all Terraform projects
[group('iac')]
plan dir:
    @mkdir -p {{ plan_dir }}
    @just _terraform {{ dir }} plan -out {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan

# Apply one or all Terraform projects
[group('iac')]
apply dir:
    @just _terraform {{ dir }} apply {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan
    rm {{ plan_dir }}/{{ replace(dir, '/', '-') }}.tfplan

# Interact with outputs one or all Terraform projects
[group('iac')]
output dir *args:
    @just _terraform {{ dir }} output {{ args }}
