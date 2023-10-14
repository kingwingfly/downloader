import Link from "next/link";

export default function BtnLink({ href, content, scoll }: { href: string, content: string, scoll?: boolean }) {
    return (
        <Link className="
        rounded-md bg-indigo-500 mx-4 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-400 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500
        "
            href={href} scroll={!scoll ? scoll : true}>{content}</Link>
    )
}