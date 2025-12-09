#!/bin/bash

# PR Review Comments Fetcher
# このスクリプトは指定されたPRのレビューコメントを取得します

set -e

# デフォルト値
PR_NUMBER=""
REPO_OWNER="UtakataKyosui"
REPO_NAME="PLocoBuf"

# 使用方法を表示
usage() {
    echo "Usage: $0 [PR_NUMBER]"
    echo ""
    echo "Options:"
    echo "  PR_NUMBER    Pull request number (optional, defaults to first open PR)"
    echo ""
    echo "Examples:"
    echo "  $0           # Get comments for the first open PR"
    echo "  $0 1         # Get comments for PR #1"
    exit 1
}

# 引数の解析
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    usage
fi

if [ -n "$1" ]; then
    PR_NUMBER="$1"
else
    # 最初のオープンPRを取得
    echo "🔍 Finding open pull requests..."
    PR_NUMBER=$(gh pr list --state open --json number --jq '.[0].number' 2>/dev/null || echo "")
    
    if [ -z "$PR_NUMBER" ]; then
        echo "❌ No open pull requests found"
        exit 1
    fi
    
    echo "✅ Found PR #${PR_NUMBER}"
fi

echo ""
echo "================================================"
echo "  PR #${PR_NUMBER} Review Comments"
echo "================================================"
echo ""

# PRの基本情報を取得
echo "📋 PR Information:"
gh pr view "${PR_NUMBER}" --json title,author,state,createdAt --jq '"Title: " + .title, "Author: " + .author.login, "State: " + .state, "Created: " + .createdAt'
echo ""

# レビューとコメントを取得
echo "💬 Fetching reviews and comments..."
echo ""

# レビューを取得
echo "=== Reviews ==="
gh pr view "${PR_NUMBER}" --json reviews --jq '.reviews[] | "---\nAuthor: " + .author.login + "\nState: " + .state + "\nSubmitted: " + .submittedAt + "\nBody:\n" + .body + "\n"'
echo ""

# 一般コメントを取得
echo "=== Comments ==="
gh pr view "${PR_NUMBER}" --json comments --jq '.comments[] | "---\nAuthor: " + .author.login + "\nCreated: " + .createdAt + "\nBody:\n" + .body + "\n"'
echo ""

# レビューコメント(コード上のコメント)を取得
echo "=== Review Comments (Code Comments) ==="
gh api "repos/${REPO_OWNER}/${REPO_NAME}/pulls/${PR_NUMBER}/comments" --jq '.[] | "---\nFile: " + .path + "\nLine: " + (.line // .original_line | tostring) + "\nAuthor: " + .user.login + "\nCreated: " + .created_at + "\nBody:\n" + .body + "\n"'

echo ""
echo "================================================"
echo "✅ Done!"
echo "================================================"
