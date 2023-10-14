export default function Input({ name, type }: { name: string, type?: string }) {
    return (
        <input name={name} type={type} autoComplete={type} required
            className="min-w-0 flex-auto rounded-md border-0 bg-white/5 px-3.5 py-2 shadow-sm ring-1 ring-inset ring-white/10 focus:ring-2 focus:ring-inset focus:ring-indigo-500 sm:text-sm sm:leading-6"
            placeholder={`Enter the ${name}`}
        />
    )
}