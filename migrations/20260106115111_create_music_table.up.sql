-- Add up migration script here
-- 音乐信息表：存储 B站音乐视频元数据
-- 设计说明：
-- - bvid 为业务唯一标识，防止重复
-- - 支持软删除（is_deleted）与喜欢标记（is_liked）
-- - 自动维护 created_at / updated_at 时间戳
-- - 索引覆盖常用查询：按作者、歌手、喜欢状态、未删除列表等

CREATE TABLE IF NOT EXISTS musics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- B站视频ID，格式如 BV1xx...，唯一业务键
    bvid TEXT NOT NULL UNIQUE CHECK (length(bvid) BETWEEN 1 AND 255),

    -- 歌曲名称
    song_name TEXT NOT NULL,

    -- 视频CID（用于获取音频流）
    cid TEXT NOT NULL CHECK (length(cid) BETWEEN 1 AND 255),

    -- 歌手名称
    songer TEXT NOT NULL,

    -- UP主/作者名称（可为空）
    author TEXT,

    -- 歌曲时长（秒），非负
    duration INTEGER DEFAULT 0 CHECK (duration >= 0),

    -- 是否喜欢：0=未喜欢，1=喜欢
    is_liked INTEGER NOT NULL DEFAULT 0 CHECK (is_liked IN (0, 1)),

    -- 软删除标记：0=正常，1=已删除
    is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1)),

    -- 创建时间
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    -- 最后更新时间
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引设计（共4个，覆盖主要查询场景）
-- 1. 按“喜欢+未删除”查询并按时间倒序（主列表页）
CREATE INDEX IF NOT EXISTS idx_musics_list ON musics(is_liked, is_deleted, created_at DESC);

-- 2. 按歌手查询（过滤已删除）
CREATE INDEX IF NOT EXISTS idx_musics_songer ON musics(songer, is_deleted);

-- 3. 按作者（UP主）查询（过滤已删除）
CREATE INDEX IF NOT EXISTS idx_musics_author ON musics(author, is_deleted);

-- 4. 纯按创建时间倒序（备用，如“最新添加”）
CREATE INDEX IF NOT EXISTS idx_musics_created_at ON musics(created_at DESC);

-- 自动更新 updated_at（要求 SQLite >= 3.35.0）
-- 如果你不确定 SQLite 版本，建议在应用层更新该字段
-- 先删除可能存在的旧触发器
DROP TRIGGER IF EXISTS update_musics_timestamp;

-- 创建兼容性触发器（适用于 SQLite 所有版本）
CREATE TRIGGER update_musics_timestamp
AFTER UPDATE ON musics
WHEN OLD.updated_at = NEW.updated_at  -- 仅当用户未手动更新 updated_at 时才触发
BEGIN
    UPDATE musics
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- 表用途说明（SQLite 不支持 COMMENT，保留为注释）
-- 用于管理用户收藏/缓存的 B站音乐视频，支持标记喜欢、软删除、快速检索。
-- 所有查询应始终携带 is_deleted = 0 条件以忽略已删除项。
